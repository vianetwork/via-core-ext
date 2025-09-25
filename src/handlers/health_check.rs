use axum::{Json, extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::state::AppState;

/// GET /health_check
pub async fn health_check_handler(State(svc): State<Arc<AppState>>) -> impl IntoResponse {
    match svc.health_check.health_check().await {
        Ok(resp) => Json(resp).into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )
            .into_response(),
    }
}
