use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::broadcast::{self, error::RecvError};
use tokio_util::sync::CancellationToken;

use crate::sse::{SseEvent, Subscription, SubscriptionKind};

pub type ConnectionId = String;
pub type SubscriptionMap = HashMap<SubscriptionKind, Subscription>;
pub type Connections = Arc<Mutex<HashMap<ConnectionId, ConnectionEntry>>>;

pub struct ConnectionEntry {
    pub account: String,
    pub subscriptions: Arc<Mutex<SubscriptionMap>>,
}

pub enum Received {
    Event(u64, Arc<SseEvent>),
    Lagged,
    Closed,
}

pub struct SseConnection {
    id: ConnectionId,
    rx: broadcast::Receiver<(u64, Arc<SseEvent>)>,
    subscriptions: Arc<Mutex<SubscriptionMap>>,
    connections: Connections,
    cancel: CancellationToken,
}

impl SseConnection {
    pub fn new(
        id: ConnectionId,
        rx: broadcast::Receiver<(u64, Arc<SseEvent>)>,
        subscriptions: Arc<Mutex<SubscriptionMap>>,
        connections: Connections,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            id,
            rx,
            subscriptions,
            connections,
            cancel,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn recv(&mut self) -> Received {
        tokio::select! {
            _ = self.cancel.cancelled() => Received::Closed,
            received = self.rx.recv() => match received {
                Ok((revision, event)) => Received::Event(revision, event),
                Err(RecvError::Lagged(_)) => Received::Lagged,
                Err(RecvError::Closed) => Received::Closed,
            },
        }
    }

    pub fn matches(&self, event: &SseEvent) -> Vec<SubscriptionKind> {
        let mut kinds: Vec<SubscriptionKind> = self
            .subscriptions
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .values()
            .filter(|sub| sub.matches(event))
            .map(Subscription::kind)
            .collect();
        kinds.sort();
        kinds
    }
}

impl Drop for SseConnection {
    fn drop(&mut self) {
        self.connections
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .remove(&self.id);
    }
}
