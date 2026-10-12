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

use crate::{
    config::Config,
    event::Event,
    helpers::random_token,
    sse::{project, SseEvent, Subscription, SubscriptionKind},
};

use super::sse_connection::{ConnectionEntry, Connections, SseConnection, SubscriptionMap};

const BROADCAST_CAPACITY: usize = 1024;
const QUEUE_CAPACITY: usize = 1024;

// The SSE sub-controller
pub struct SseController {
    tx: broadcast::Sender<(u64, Arc<SseEvent>)>,
    // todo: see if we can remove once we track file versions
    revision: AtomicU64,
    cancel: CancellationToken,
    connections: Connections,
    queue_tx: mpsc::Sender<Event>,
}

impl SseController {
    pub fn new(cancel: CancellationToken) -> (Arc<Self>, mpsc::Receiver<Event>) {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let (queue_tx, queue_rx) = mpsc::channel(QUEUE_CAPACITY);

        let controller = Arc::new(Self {
            tx,
            revision: AtomicU64::new(0),
            cancel,
            connections: Arc::new(Mutex::new(HashMap::new())),
            queue_tx,
        });

        (controller, queue_rx)
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::SeqCst)
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
        let revision = self.revision.fetch_add(1, Ordering::SeqCst) + 1;
        let _ = self.tx.send((revision, Arc::new(event)));
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

    fn with_subscriptions<T>(
        &self,
        connection_id: &str,
        account: &str,
        f: impl FnOnce(&mut SubscriptionMap) -> T,
    ) -> Option<T> {
        let connections = self
            .connections
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let Some(entry) = connections.get(connection_id) else {
            debug!("unknown sse connection");
            return None;
        };
        if entry.account != account {
            warn!("rejected sse subscription change from a different account");
            return None;
        }

        let mut subscriptions = entry
            .subscriptions
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        Some(f(&mut subscriptions))
    }

    pub fn subscribe(&self, connection_id: &str, account: &str, sub: Subscription) -> Option<u64> {
        self.with_subscriptions(connection_id, account, |subscriptions| {
            subscriptions.insert(sub.kind(), sub);
            self.revision()
        })
    }

    pub fn unsubscribe(&self, connection_id: &str, account: &str, kind: SubscriptionKind) -> bool {
        self.with_subscriptions(connection_id, account, |subscriptions| {
            subscriptions.remove(&kind);
        })
        .is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        controller::Received,
        sse::sse_event::{EntryChange, SsePayload},
    };

    fn sse_controller() -> Arc<SseController> {
        SseController::new(CancellationToken::new()).0
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
    async fn publish_reaches_connection_with_revision() {
        let sse = sse_controller();
        let mut conn = sse.connect("holden");
        assert_eq!(sse.subscribe(conn.id(), "holden", dir("docs")), Some(0));

        sse.publish(created("docs/a.txt"));
        assert_eq!(sse.revision(), 1);

        let Received::Event(revision, event) = conn.recv().await else {
            panic!("expected an event");
        };
        assert_eq!(revision, 1);
        assert_eq!(conn.matches(&event), vec![SubscriptionKind::Dir]);
    }

    #[tokio::test]
    async fn subscribing_replaces_same_kind() {
        let sse = sse_controller();
        let conn = sse.connect("holden");
        sse.subscribe(conn.id(), "holden", dir("docs"));
        sse.subscribe(conn.id(), "holden", dir("photos"));

        assert!(conn.matches(&created("docs/a.txt")).is_empty());
        assert_eq!(
            conn.matches(&created("photos/a.txt")),
            vec![SubscriptionKind::Dir]
        );
    }

    #[tokio::test]
    async fn subscribe_rejects_other_accounts_and_unknown_ids() {
        let sse = sse_controller();
        let conn = sse.connect("holden");
        assert_eq!(sse.subscribe(conn.id(), "naomi", dir("docs")), None);
        assert_eq!(sse.subscribe("unknown", "holden", dir("docs")), None);

        assert!(conn.matches(&created("docs/a.txt")).is_empty());
    }

    #[tokio::test]
    async fn unsubscribe_removes_only_that_kind() {
        let sse = sse_controller();
        let conn = sse.connect("holden");
        sse.subscribe(conn.id(), "holden", dir("docs"));
        sse.subscribe(
            conn.id(),
            "holden",
            Subscription::Tree {
                space: "rocinante".to_owned(),
            },
        );

        assert!(!sse.unsubscribe(conn.id(), "naomi", SubscriptionKind::Dir));
        assert!(sse.unsubscribe(conn.id(), "holden", SubscriptionKind::Dir));
        assert_eq!(
            conn.matches(&created("docs/a.txt")),
            vec![SubscriptionKind::Tree]
        );
    }

    #[tokio::test]
    async fn dropping_a_connection_deregisters_it() {
        let sse = sse_controller();
        let conn = sse.connect("holden");
        let id = conn.id().to_owned();
        assert!(sse.connections.lock().unwrap().contains_key(&id));

        drop(conn);
        assert!(!sse.connections.lock().unwrap().contains_key(&id));
    }

    #[tokio::test]
    async fn lagging_connection_is_told_to_reset() {
        let sse = sse_controller();
        let mut conn = sse.connect("holden");
        for i in 0..=BROADCAST_CAPACITY {
            sse.publish(created(&format!("docs/{i}.txt")));
        }

        assert!(matches!(conn.recv().await, Received::Lagged));
    }

    #[tokio::test]
    async fn cancel_closes_connections() {
        let cancel = CancellationToken::new();
        let (sse, _) = SseController::new(cancel.clone());
        let mut conn = sse.connect("holden");
        cancel.cancel();

        assert!(matches!(conn.recv().await, Received::Closed));
    }
}
