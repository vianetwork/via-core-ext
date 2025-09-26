use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    clients::da_clients::make_da_client,
    config::Config,
    handlers::{
        da::{dispatch_handler, inclusion_handler},
        health_check::health_check_handler,
    },
    services::{da::DaSvc, health_check::HealthCheckSvc},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub health_check: HealthCheckSvc,
    pub da_svc: Arc<DaSvc>,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let da_client = make_da_client(config.clone()).await?;

        // Services
        let health_check = HealthCheckSvc::new(da_client.clone());
        let da_svc = Arc::new(DaSvc::new(da_client));

        Ok(Self {
            config,
            da_svc,
            health_check,
        })
    }

    pub fn into_router(self) -> Router {
        Router::new()
            .route("/da/dispatch", post(dispatch_handler))
            .route("/da/inclusion/:blob_id", get(inclusion_handler))
            .route("/health", get(health_check_handler))
            .with_state(self.into())
    }
}
