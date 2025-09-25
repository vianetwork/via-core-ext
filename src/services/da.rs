use tokio::time::Instant;

use crate::{
    clients::da_clients::{
        DataAvailabilityClient,
        types::{DispatchResponse, InclusionData},
    },
    services::metrics::DA_METRICS,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DaSvc {
    da_client: Arc<dyn DataAvailabilityClient + Send + Sync>,
}

impl DaSvc {
    pub fn new(da_client: Arc<dyn DataAvailabilityClient + Send + Sync>) -> Self {
        Self { da_client }
    }

    /// Dispatches a blob to the data availability layer.
    pub async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> anyhow::Result<DispatchResponse> {
        let start = Instant::now();
        let response = self.da_client.dispatch_blob(batch_number, data).await?;

        DA_METRICS.dispatched_blobs.inc();
        DA_METRICS.dispatch_latency.observe(start.elapsed());

        Ok(response)
    }

    /// Fetches the inclusion data for a given blob_id.
    pub async fn get_inclusion_data(&self, blob_id: &str) -> anyhow::Result<Option<InclusionData>> {
        let response = self.da_client.get_inclusion_data(blob_id).await?;

        DA_METRICS.inclusion_queries.inc();

        Ok(response)
    }
}
