use serde::Serialize;

use crate::files::Entry;

use super::sse_subscription::SubscriptionKind;

#[derive(Serialize)]
pub struct EntryChange {
    pub space: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Entry>,
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum SsePayload {
    #[serde(rename = "entry.created")]
    EntryCreated(EntryChange),
    #[serde(rename = "entry.modified")]
    EntryModified(EntryChange),
    #[serde(rename = "entry.removed")]
    EntryRemoved(EntryChange),
    #[serde(rename = "entry.moved")]
    EntryMoved(EntryChange),
    #[serde(rename = "stream.connected")]
    StreamConnected { connection_id: String },
    #[serde(rename = "stream.reset")]
    StreamReset,
}

#[derive(Serialize)]
pub struct SseEvent {
    #[serde(flatten)]
    pub payload: SsePayload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
}

impl SseEvent {
    pub fn connected(connection_id: impl Into<String>) -> Self {
        Self {
            payload: SsePayload::StreamConnected {
                connection_id: connection_id.into(),
            },
            actor: None,
        }
    }

    pub fn reset() -> Self {
        Self {
            payload: SsePayload::StreamReset,
            actor: None,
        }
    }

    pub fn entry_change(&self) -> Option<&EntryChange> {
        match &self.payload {
            SsePayload::EntryCreated(change)
            | SsePayload::EntryModified(change)
            | SsePayload::EntryRemoved(change)
            | SsePayload::EntryMoved(change) => Some(change),
            SsePayload::StreamConnected { .. } | SsePayload::StreamReset => None,
        }
    }
}

#[derive(Serialize)]
pub struct SseFrame<'a> {
    #[serde(flatten)]
    pub event: &'a SseEvent,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub subscriptions: Vec<SubscriptionKind>,
}

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value};

    use super::*;

    fn change(path: &str, from: Option<&str>) -> EntryChange {
        EntryChange {
            space: "rocinante".to_owned(),
            path: path.to_owned(),
            from: from.map(str::to_owned),
            entry: None,
        }
    }

    fn frame(event: &SseEvent, subscriptions: Vec<SubscriptionKind>) -> serde_json::Value {
        to_value(SseFrame {
            event,
            subscriptions,
        })
        .unwrap()
    }

    #[test]
    fn entry_moved_frame_shape() {
        let event = SseEvent {
            payload: SsePayload::EntryMoved(change("docs/b.txt", Some("docs/a.txt"))),
            actor: Some("holden".to_owned()),
        };

        assert_eq!(
            frame(&event, vec![SubscriptionKind::Dir, SubscriptionKind::Tree]),
            json!({
                "kind": "entry.moved",
                "data": { "space": "rocinante", "path": "docs/b.txt", "from": "docs/a.txt" },
                "actor": "holden",
                "subscriptions": ["dir", "tree"],
            })
        );
    }

    #[test]
    fn entry_kinds_are_dotted() {
        let kinds = [
            (SsePayload::EntryCreated(change("a", None)), "entry.created"),
            (
                SsePayload::EntryModified(change("a", None)),
                "entry.modified",
            ),
            (SsePayload::EntryRemoved(change("a", None)), "entry.removed"),
        ];

        for (payload, kind) in kinds {
            let event = SseEvent {
                payload,
                actor: None,
            };
            assert_eq!(
                frame(&event, vec![SubscriptionKind::Shares]),
                json!({
                    "kind": kind,
                    "data": { "space": "rocinante", "path": "a" },
                    "subscriptions": ["shares"],
                })
            );
        }
    }

    #[test]
    fn stream_frames_shape() {
        assert_eq!(
            frame(&SseEvent::connected("k3J"), vec![]),
            json!({ "kind": "stream.connected", "data": { "connection_id": "k3J" } })
        );
        assert_eq!(
            frame(&SseEvent::reset(), vec![]),
            json!({ "kind": "stream.reset" })
        );
    }
}
