pub mod celestia;
pub mod in_memory;
pub mod types;

use std::{fmt, sync::Arc};

use async_trait::async_trait;
use types::{DAError, DispatchResponse, InclusionData};

use crate::{
    clients::da_clients::{celestia::CelestiaClient, in_memory::InMemoryClient},
    config::{Config, DaBackend},
};

pub async fn make_da_client(
    config: Config,
) -> anyhow::Result<Arc<dyn DataAvailabilityClient + Send + Sync>> {
    match config.da_backend {
        DaBackend::Celestia => {
            let client = CelestiaClient::new(
                config.da_node_url.unwrap(),
                config.da_auth_token.unwrap(),
                config.da_blob_size_limit,
            )
            .await?;
            Ok(Arc::new(client))
        }

        DaBackend::InMemory => Ok(Arc::new(InMemoryClient::new(config.da_blob_size_limit))),
    }
}

/// Trait that defines the interface for the data availability layer clients.
#[async_trait]
pub trait DataAvailabilityClient: Sync + Send + fmt::Debug {
    /// Dispatches a blob to the data availability layer.
    async fn dispatch_blob(
        &self,
        batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError>;

    /// Fetches the inclusion data for a given blob_id.
    async fn get_inclusion_data(&self, blob_id: &str) -> Result<Option<InclusionData>, DAError>;

    /// Clones the client and wraps it in a Box.
    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient>;

    /// Returns the maximum size of the blob (in bytes) that can be dispatched. None means no limit.
    fn blob_size_limit(&self) -> Option<usize>;

    /// Ping the DA layer.
    async fn ping(&self) -> anyhow::Result<bool>;
}

impl Clone for Box<dyn DataAvailabilityClient> {
    fn clone(&self) -> Box<dyn DataAvailabilityClient> {
        self.clone_boxed()
    }
}
