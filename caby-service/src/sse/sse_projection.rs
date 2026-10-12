use tracing::warn;

use crate::{
    config::Config,
    event::{Event, EventKind},
    files::{try_build_entry, Entry},
    space::Space,
};

use super::sse_event::{EntryChange, SseEvent, SsePayload};

async fn try_project_entry(cfg: &Config, space: &Space, event: &Event) -> Option<Entry> {
    match try_build_entry(cfg, space, &event.path).await {
        Ok(entry) => entry,
        Err(err) => {
            warn!(
                "could not build entry for {}/{}: {:#}",
                event.space,
                event.path.display(),
                err
            );
            None
        }
    }
}

pub async fn project(cfg: &Config, event: &Event) -> Vec<SseEvent> {
    let Some(space) = cfg.find_space(&event.space) else {
        warn!("dropping sse projection for unknown space {}", event.space);
        return vec![];
    };

    let entry = match event.kind {
        EventKind::FileRemoved => None,
        _ => try_project_entry(cfg, &space, event).await,
    };

    let from = match &event.kind {
        EventKind::FileMoved { from } => Some(from.to_string_lossy().into_owned()),
        _ => None,
    };

    let change = EntryChange {
        space: event.space.clone(),
        path: event.path.to_string_lossy().into_owned(),
        from,
        entry,
    };

    let payload = match event.kind {
        EventKind::FileCreated => SsePayload::EntryCreated(change),
        EventKind::FileModified => SsePayload::EntryModified(change),
        EventKind::FileRemoved => SsePayload::EntryRemoved(change),
        EventKind::FileMoved { .. } => SsePayload::EntryMoved(change),
    };

    vec![SseEvent {
        payload,
        actor: event.actor.clone(),
    }]
}
