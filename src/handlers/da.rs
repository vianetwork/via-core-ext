use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct DispatchRequest {
    pub batch_number: u32,
    pub data: String,
}

#[derive(Serialize)]
pub struct InclusionResponse {
    pub data: String,
}

/// POST /dispatch
pub async fn dispatch_handler(
    State(svc): State<Arc<AppState>>,
    payload: Result<Json<DispatchRequest>, JsonRejection>,
) -> impl IntoResponse {
    let payload = match payload {
        Ok(Json(p)) => p,
        Err(err) => {
            tracing::error!("Invalid JSON: {}", err);
            return (StatusCode::BAD_REQUEST, "Invalid JSON body").into_response();
        }
    };

    let data = match hex::decode(payload.data) {
        Ok(data) => data,
        Err(_) => {
            tracing::error!("Invalid data format");
            return (
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid data format, must be a hex string".to_string(),
            )
                .into_response();
        }
    };

    match svc.da_svc.dispatch_blob(payload.batch_number, data).await {
        Ok(resp) => Json(resp).into_response(),
        Err(err) => {
            tracing::error!("Error to dispatch the blob data: {}", err);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error to dispatch the blob data: {}", err),
            )
                .into_response()
        }
    }
}

/// GET /inclusion/:blob_id
pub async fn inclusion_handler(
    State(svc): State<Arc<AppState>>,
    Path(blob_id): Path<String>,
) -> impl IntoResponse {
    match svc.da_svc.get_inclusion_data(&blob_id).await {
        Ok(Some(data)) => Json(InclusionResponse {
            data: hex::encode(&data.data),
        })
        .into_response(),
        Ok(None) => axum::http::StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            tracing::error!("Error to fetch blob data: {}", err.root_cause());
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error to fetch blob data: {}", err),
            )
                .into_response()
        }
    }
}
