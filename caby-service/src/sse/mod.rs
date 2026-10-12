use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use crate::{config::Config, event::Event, helpers::random_token};

pub mod sse_connection;
pub mod sse_event;
mod sse_projection;
pub mod sse_subscription;
pub mod sse_token;

pub use sse_connection::{Received, SseConnection};
pub use sse_event::{SseEvent, SseFrame};
pub use sse_subscription::{Subscription, SubscriptionKind};

use sse_connection::{ConnectionEntry, Connections};
use sse_projection::project;

const BROADCAST_CAPACITY: usize = 1024;
const QUEUE_CAPACITY: usize = 1024;

pub struct SseHub {
    tx: broadcast::Sender<(u64, Arc<SseEvent>)>,
    seq: AtomicU64,
    cancel: CancellationToken,
    connections: Connections,
    queue_tx: mpsc::Sender<Event>,
}

impl SseHub {
    pub fn new(cancel: CancellationToken) -> (Arc<Self>, mpsc::Receiver<Event>) {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let (queue_tx, queue_rx) = mpsc::channel(QUEUE_CAPACITY);

        let hub = Arc::new(Self {
            tx,
            seq: AtomicU64::new(0),
            cancel,
            connections: Arc::new(Mutex::new(HashMap::new())),
            queue_tx,
        });

        (hub, queue_rx)
    }

    pub fn seq(&self) -> u64 {
        self.seq.load(Ordering::SeqCst)
    }

    pub fn forward(&self, event: &Event) {
        if self.queue_tx.try_send(event.clone()).is_err() {
            warn!(
                "dropped sse event for {}/{}",
                event.space,
                event.path.display()
            );
        }
    }

    pub fn publish(&self, event: SseEvent) {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        let _ = self.tx.send((seq, Arc::new(event)));
    }

    pub async fn run(self: Arc<Self>, cfg: Config, mut queue_rx: mpsc::Receiver<Event>) {
        loop {
            let event = tokio::select! {
                _ = self.cancel.cancelled() => break,
                event = queue_rx.recv() => match event {
                    Some(event) => event,
                    None => break,
                },
            };

            for sse_event in project(&cfg, &event).await {
                self.publish(sse_event);
            }
        }
    }

    pub fn connect(&self, account: &str) -> SseConnection {
        let id = random_token();
        let subscriptions = Arc::new(Mutex::new(HashMap::new()));

        self.connections
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .insert(
                id.clone(),
                ConnectionEntry {
                    account: account.to_owned(),
                    subscriptions: subscriptions.clone(),
                },
            );

        SseConnection::new(
            id,
            self.tx.subscribe(),
            subscriptions,
            self.connections.clone(),
            self.cancel.clone(),
        )
    }

    pub fn subscribe(&self, connection_id: Option<&str>, account: &str, sub: Subscription) {
        let Some(connection_id) = connection_id else {
            return;
        };

        let connections = self
            .connections
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let Some(entry) = connections.get(connection_id) else {
            debug!("ignoring subscription for unknown sse connection");
            return;
        };
        if entry.account != account {
            warn!("rejected sse subscription from a different account");
            return;
        }

        entry
            .subscriptions
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .insert(sub.kind(), sub);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use sse_event::{EntryChange, SsePayload};

    fn hub() -> Arc<SseHub> {
        SseHub::new(CancellationToken::new()).0
    }

    fn dir(path: &str) -> Subscription {
        Subscription::Dir {
            space: "rocinante".to_owned(),
            path: PathBuf::from(path),
        }
    }

    fn created(path: &str) -> SseEvent {
        SseEvent {
            payload: SsePayload::EntryCreated(EntryChange {
                space: "rocinante".to_owned(),
                path: path.to_owned(),
                from: None,
                entry: None,
            }),
            actor: None,
        }
    }

    #[tokio::test]
    async fn publish_reaches_connection_with_seq() {
        let hub = hub();
        let mut conn = hub.connect("holden");
        hub.subscribe(Some(conn.id()), "holden", dir("docs"));

        hub.publish(created("docs/a.txt"));
        assert_eq!(hub.seq(), 1);

        let Received::Event(seq, event) = conn.recv().await else {
            panic!("expected an event");
        };
        assert_eq!(seq, 1);
        assert_eq!(conn.matches(&event), vec![SubscriptionKind::Dir]);
    }

    #[tokio::test]
    async fn subscribing_replaces_same_kind() {
        let hub = hub();
        let conn = hub.connect("holden");
        hub.subscribe(Some(conn.id()), "holden", dir("docs"));
        hub.subscribe(Some(conn.id()), "holden", dir("photos"));

        assert!(conn.matches(&created("docs/a.txt")).is_empty());
        assert_eq!(
            conn.matches(&created("photos/a.txt")),
            vec![SubscriptionKind::Dir]
        );
    }

    #[tokio::test]
    async fn subscribe_rejects_other_accounts_and_unknown_ids() {
        let hub = hub();
        let conn = hub.connect("holden");
        hub.subscribe(Some(conn.id()), "naomi", dir("docs"));
        hub.subscribe(Some("unknown"), "holden", dir("docs"));
        hub.subscribe(None, "holden", dir("docs"));

        assert!(conn.matches(&created("docs/a.txt")).is_empty());
    }

    #[tokio::test]
    async fn dropping_a_connection_deregisters_it() {
        let hub = hub();
        let conn = hub.connect("holden");
        let id = conn.id().to_owned();
        assert!(hub.connections.lock().unwrap().contains_key(&id));

        drop(conn);
        assert!(!hub.connections.lock().unwrap().contains_key(&id));
    }

    #[tokio::test]
    async fn lagging_connection_is_told_to_reset() {
        let hub = hub();
        let mut conn = hub.connect("holden");
        for i in 0..=BROADCAST_CAPACITY {
            hub.publish(created(&format!("docs/{i}.txt")));
        }

        assert!(matches!(conn.recv().await, Received::Lagged));
    }

    #[tokio::test]
    async fn cancel_closes_connections() {
        let cancel = CancellationToken::new();
        let (hub, _) = SseHub::new(cancel.clone());
        let mut conn = hub.connect("holden");
        cancel.cancel();

        assert!(matches!(conn.recv().await, Received::Closed));
    }
}
