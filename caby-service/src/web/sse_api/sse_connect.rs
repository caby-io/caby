use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::{Query, State},
    http::{HeaderName, HeaderValue, StatusCode},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Response, Sse,
    },
};
use futures_util::stream::{self, Stream, StreamExt};
use serde::Deserialize;
use tracing::warn;

use crate::{
    config::Config,
    controller::{Received, SseConnection, SseController},
    jsend::JSendBuilder,
    sse::{sse_token, SseEvent, SseFrame, SubscriptionKind},
};

static X_ACCEL_BUFFERING: HeaderName = HeaderName::from_static("x-accel-buffering");

#[derive(Deserialize)]
pub struct SseConnectQuery {
    token: Option<String>,
}

fn into_event(
    revision: Option<u64>,
    event: &SseEvent,
    subscriptions: Vec<SubscriptionKind>,
) -> Option<Event> {
    let frame = SseFrame {
        event,
        subscriptions,
    };
    let sse_event = match Event::default().json_data(&frame) {
        Ok(e) => e,
        Err(err) => {
            warn!("could not serialize sse frame: {:#}", err);
            return None;
        }
    };

    Some(match revision {
        Some(revision) => sse_event.id(revision.to_string()),
        None => sse_event,
    })
}

async fn next_event(conn: &mut SseConnection) -> Option<Event> {
    loop {
        match conn.recv().await {
            Received::Event(revision, event) => {
                let subscriptions = conn.matches(&event);
                if subscriptions.is_empty() {
                    continue;
                }
                if let Some(sse_event) = into_event(Some(revision), &event, subscriptions) {
                    return Some(sse_event);
                }
            }
            Received::Lagged => {
                if let Some(sse_event) = into_event(None, &SseEvent::reset(), vec![]) {
                    return Some(sse_event);
                }
            }
            Received::Closed => return None,
        }
    }
}

fn event_stream(conn: SseConnection) -> impl Stream<Item = Result<Event, Infallible>> {
    let connected = into_event(None, &SseEvent::connected(conn.id()), vec![]);

    stream::iter(connected)
        .chain(stream::unfold(conn, |mut conn| async move {
            next_event(&mut conn).await.map(|event| (event, conn))
        }))
        .map(Ok)
}

pub async fn handle_connect_sse(
    State(cfg): State<Config>,
    State(sse): State<Arc<SseController>>,
    Query(query): Query<SseConnectQuery>,
) -> Response {
    let unauthorized_resp = || {
        JSendBuilder::new()
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("unauthorized")
            .into_response()
    };

    let Some(token) = query.token else {
        return unauthorized_resp();
    };
    let payload = match sse_token::decode_token(&cfg, &token) {
        Ok(p) => p,
        Err(err) => {
            warn!("could not decode sse token: {:#}", err);
            return unauthorized_resp();
        }
    };
    if payload.is_expired() {
        return unauthorized_resp();
    }

    let conn = sse.connect(&payload.account);

    (
        [(X_ACCEL_BUFFERING.clone(), HeaderValue::from_static("no"))],
        Sse::new(event_stream(conn)).keep_alive(KeepAlive::default()),
    )
        .into_response()
}
