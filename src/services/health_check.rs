use std::sync::Arc;

use crate::{
    clients::da_clients::DataAvailabilityClient,
    types::health_check::{HealthCheckResponse, ServiceStatus},
};

#[derive(Debug, Clone)]
pub struct HealthCheckSvc {
    da_client: Arc<dyn DataAvailabilityClient + Send + Sync>,
}

impl HealthCheckSvc {
    pub fn new(da_client: Arc<dyn DataAvailabilityClient + Send + Sync>) -> Self {
        Self { da_client }
    }

    pub async fn health_check(&self) -> anyhow::Result<HealthCheckResponse> {
        let da = ServiceStatus {
            status: self.da_client.ping().await?,
            message: "Data availability is healthy".to_string(),
        };

        Ok(HealthCheckResponse { da })
    }
}
