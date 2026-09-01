use crate::{
    event::{emit, Event, Sender},
    files::ops::{
        create_dir, is_name_too_long, write_file, DirConflictStrategy, FileConflictStrategy,
        WriteOutcome,
    },
    jsend::JSendBuilder,
    space::Space,
    web::{extractors::RequireAccount, files_api::files_list::FilesPathParams},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::path::PathBuf;
use tracing::error;

#[derive(Deserialize)]
#[serde(tag = "entry_type", rename_all = "lowercase")]
pub enum PutEntryRequest {
    Directory {
        name: String,
        #[serde(default)]
        conflict_strategy: DirConflictStrategy,
    },
    File {
        name: String,
        // todo: this probably isnt the best for raw files?
        content: Option<String>,
        conflict_strategy: FileConflictStrategy,
    },
}

// used to create directories and small, inline, files
pub async fn handle_put_files(
    State(events_tx): State<Sender>,
    space: Space,
    RequireAccount(account): RequireAccount,
    path_params: Path<FilesPathParams>,
    Json(payload): Json<PutEntryRequest>,
) -> Response {
    let rel_path = path_params
        .file_path
        .clone()
        .map_or(PathBuf::from(""), PathBuf::from);

    match payload {
        PutEntryRequest::Directory {
            name,
            conflict_strategy,
        } => put_directory(&space, &rel_path, &name, conflict_strategy).await,
        PutEntryRequest::File {
            name,
            content,
            conflict_strategy,
        } => {
            put_file(
                &space,
                &events_tx,
                &account.name,
                &rel_path,
                &name,
                content.as_deref(),
                conflict_strategy,
            )
            .await
        }
    }
}

async fn put_directory(
    space: &Space,
    rel_path: &std::path::Path,
    name: &str,
    conflict_strategy: DirConflictStrategy,
) -> Response {
    let resp = JSendBuilder::new();
    let rel = rel_path.join(name);

    match create_dir(space, &rel, conflict_strategy).await {
        Ok(path) => resp
            .status_code(StatusCode::CREATED)
            .success(path.to_string_lossy().into_owned())
            .into_response(),
        Err(err) if is_name_too_long(&err) => resp
            .status_code(StatusCode::BAD_REQUEST)
            .fail("name exceeds the maximum length")
            .into_response(),
        Err(err) => {
            error!("could not create dir at {:?}: {:#}", rel, err);
            resp.fail("could not create directory").into_response()
        }
    }
}

async fn put_file(
    space: &Space,
    events_tx: &Sender,
    author: &str,
    rel_path: &std::path::Path,
    name: &str,
    content: Option<&str>,
    conflict_strategy: FileConflictStrategy,
) -> Response {
    let resp = JSendBuilder::new();
    let rel = rel_path.join(name);
    let content = content.unwrap_or_default();

    let outcome = match write_file(space, &rel, content, conflict_strategy).await {
        Ok(outcome) => outcome,
        Err(err) if is_name_too_long(&err) => {
            return resp
                .status_code(StatusCode::BAD_REQUEST)
                .fail("name exceeds the maximum length")
                .into_response();
        }
        Err(err) => {
            error!("could not write file at {:?}: {:#}", rel, err);
            return resp.fail("could not write file").into_response();
        }
    };

    let resp = match outcome {
        WriteOutcome::Created(path) | WriteOutcome::Deconflicted(path) => {
            emit(
                events_tx,
                Event::from_create(space.name.clone(), path.clone()).by(author),
            );
            resp.status_code(StatusCode::CREATED)
                .success(path.to_string_lossy().into_owned())
        }
        WriteOutcome::Overwritten(path) => {
            emit(
                events_tx,
                Event::from_modify(space.name.clone(), path.clone()).by(author),
            );
            resp.status_code(StatusCode::OK)
                .success(path.to_string_lossy().into_owned())
        }
        WriteOutcome::Skipped(path) => resp
            .status_code(StatusCode::OK)
            .success(path.to_string_lossy().into_owned()),
    };

    resp.into_response()
}
