use axum::{
    Json,
    extract::{Path, State},
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
    Json(payload): Json<DispatchRequest>,
) -> impl IntoResponse {
    let data = match hex::decode(payload.data) {
        Ok(data) => data,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid data format".to_string(),
            )
                .into_response();
        }
    };

    match svc.da_svc.dispatch_blob(payload.batch_number, data).await {
        Ok(resp) => Json(resp).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
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
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
