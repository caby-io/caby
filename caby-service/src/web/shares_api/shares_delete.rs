use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::warn;

use crate::{
    auth::AuthUser, jsend::JSendBuilder, share::Share, space::Space, web::shares_api::ShareIdParam,
};

#[derive(Serialize)]
struct DeleteShareResponse {
    id: String,
}

pub async fn handle_delete_share(
    space: Space,
    auth: AuthUser,
    Path(params): Path<ShareIdParam>,
) -> Response {
    let resp = JSendBuilder::new();

    if auth.as_account().is_none() {
        return resp
            .status_code(StatusCode::FORBIDDEN)
            .fail("only accounts can delete shares")
            .into_response();
    }

    let share = match Share::load(&space, &params.id).await {
        Ok(Some(share)) => share,
        Ok(None) => {
            return resp
                .status_code(StatusCode::NOT_FOUND)
                .fail("share not found")
                .into_response()
        }
        Err(err) => {
            warn!("could not load share {}: {:#}", params.id, err);
            return resp.internal_error().into_response();
        }
    };

    if let Err(err) = Share::delete(&space, &share.id).await {
        warn!("could not delete share {}: {:#}", share.id, err);
        return resp.internal_error().into_response();
    }

    resp.success(DeleteShareResponse { id: share.id })
        .into_response()
}
