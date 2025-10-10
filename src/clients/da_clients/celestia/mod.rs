use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use anyhow::anyhow;
use async_trait::async_trait;
use celestia_rpc::{BlobClient, Client, HeaderClient, P2PClient, TxConfig};
use celestia_types::{
    AppVersion, Blob, Commitment, consts::appconsts::SHARE_VERSION_ZERO, nmt::Namespace,
};
use hex;

use crate::clients::da_clients::{
    DataAvailabilityClient,
    types::{DAError, DispatchResponse, InclusionData},
};

/// If no value is provided for GasPrice, then this will be serialized to `-1.0` which means the node that
/// receives the request will calculate the GasPrice for given blob.
const GAS_PRICE: f64 = -1.0;

/// An implementation of the `DataAvailabilityClient` trait that stores the pubdata in Celestia DA.
#[derive(Clone)]
pub struct CelestiaClient {
    light_node_url: String,
    client: Arc<Client>,
    blob_size_limit: usize,
    namespace: Namespace,
    app_version: AppVersion,
}

impl CelestiaClient {
    pub async fn new(
        node_url: String,
        auth_token: String,
        blob_size_limit: usize,
    ) -> anyhow::Result<Self> {
        let client = Client::new(&node_url, Some(&auth_token))
            .await
            .map_err(|error| anyhow!("Failed to create a client: {error}"))?;

        // Ensure connectivity by calling P2P info
        client.p2p_info().await?;

        let mut namespace_bytes = [0u8; 8];
        namespace_bytes[..3].copy_from_slice(b"VIA");

        let namespace = Namespace::new_v0(&namespace_bytes).map_err(|error| DAError {
            error: error.into(),
            is_retriable: false,
        })?;

        Ok(Self {
            light_node_url: node_url,
            client: Arc::new(client),
            blob_size_limit,
            namespace,
            app_version: AppVersion::V5,
        })
    }
}

#[async_trait]
impl DataAvailabilityClient for CelestiaClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError> {
        let blob =
            Blob::new(self.namespace, data.clone(), None, self.app_version).map_err(|error| {
                DAError {
                    error: error.into(),
                    is_retriable: false,
                }
            })?;

        let commitment = Commitment::from_blob(
            self.namespace,
            &data,
            SHARE_VERSION_ZERO,
            None,
            self.app_version,
        )
        .map_err(|error| DAError {
            error: anyhow!("Error to create commitment: {}", error.to_string()),
            is_retriable: false,
        })?;

        let tx_config = TxConfig {
            gas_price: Some(GAS_PRICE),
            ..Default::default()
        };

        let block_height = self
            .client
            .blob_submit(&[blob], tx_config)
            .await
            .map_err(|error| DAError {
                error: anyhow!("Error to submit blob: {}", error.to_string()),
                is_retriable: true,
            })?;

        // Construct blob_id = [block_height (8 bytes) | commitment hash (32 bytes)]
        let mut blob_id = Vec::with_capacity(8 + 32);
        blob_id.extend_from_slice(&block_height.to_be_bytes());
        blob_id.extend_from_slice(commitment.hash());

        Ok(DispatchResponse {
            blob_id: hex::encode(blob_id),
        })
    }

    async fn get_inclusion_data(&self, blob_id: &str) -> Result<Option<InclusionData>, DAError> {
        let blob_id_bytes = hex::decode(blob_id).map_err(|error| DAError {
            error: error.into(),
            is_retriable: false,
        })?;

        let block_height =
            u64::from_be_bytes(blob_id_bytes[..8].try_into().map_err(|_| DAError {
                error: anyhow!("Failed to convert block height"),
                is_retriable: false,
            })?);

        let commitment_data: [u8; 32] = blob_id_bytes[8..40].try_into().map_err(|_| DAError {
            error: anyhow!("Failed to convert commitment"),
            is_retriable: false,
        })?;

        let blob = self
            .client
            .blob_get(
                block_height,
                self.namespace,
                Commitment::new(commitment_data),
            )
            .await
            .map_err(|error| DAError {
                error: anyhow!("Error to get blob: {}", error.to_string()),
                is_retriable: true,
            })?;

        Ok(Some(InclusionData { data: blob.data }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        Some(self.blob_size_limit)
    }

    async fn ping(&self) -> anyhow::Result<bool> {
        match self.client.header_network_head().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Debug for CelestiaClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CelestiaClient")
            .field("light_node_url", &self.light_node_url)
            .finish()
    }
}
