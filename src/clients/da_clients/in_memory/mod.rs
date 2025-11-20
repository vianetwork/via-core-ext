use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use async_trait::async_trait;
use celestia_types::consts::appconsts::SHARE_VERSION_ZERO;
use celestia_types::nmt::Namespace;
use celestia_types::{AppVersion, Commitment};

use crate::clients::da_clients::common::VIA_NAME_SPACE_BYTES;
use crate::clients::da_clients::types::{ViaDaBlob, deserialize_blob_ids};
use crate::clients::da_clients::{
    DataAvailabilityClient,
    types::{DAError, DispatchResponse, InclusionData},
};

#[derive(Clone, Debug)]
pub struct InMemoryClient {
    storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    blob_size_limit: usize,
    namespace: Namespace,
}

impl InMemoryClient {
    pub fn new(blob_size_limit: usize) -> anyhow::Result<Self> {
        let namespace = Namespace::new_v0(&VIA_NAME_SPACE_BYTES)?;

        Ok(Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            blob_size_limit,
            namespace,
        })
    }
}

#[async_trait]
impl DataAvailabilityClient for InMemoryClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError> {
        let commitment = Commitment::from_blob(
            self.namespace,
            &data,
            SHARE_VERSION_ZERO,
            None,
            AppVersion::V5,
        )
        .map_err(|error| DAError {
            error: anyhow!("Error to create commitment: {}", error.to_string()),
            is_retriable: false,
        })?;

        // Construct blob_id = [block_height (8 bytes) | commitment hash (32 bytes)]
        let mut blob_id_bytes = Vec::with_capacity(8 + 32);
        blob_id_bytes.extend_from_slice(&0u64.to_be_bytes());
        blob_id_bytes.extend_from_slice(commitment.hash());

        let blob_id = hex::encode(&blob_id_bytes);

        self.storage.lock().unwrap().insert(blob_id.clone(), data);

        Ok(DispatchResponse { blob_id })
    }

    async fn get_inclusion_data(&self, blob_id: &str) -> Result<Option<InclusionData>, DAError> {
        let storage = self.storage.lock().unwrap();

        let Some(blob) = storage
            .get(blob_id)
            .map(|data| InclusionData { data: data.clone() })
        else {
            return Ok(None);
        };

        let data = match ViaDaBlob::from_bytes(&blob.data) {
            Some(blob) => {
                if blob.chunks == 1 {
                    blob.data
                } else {
                    let blob_ids = deserialize_blob_ids(&blob.data).map_err(|_| DAError {
                        error: anyhow!("Failed to deserialize blob ids"),
                        is_retriable: false,
                    })?;
                    if blob_ids.len() != blob.chunks {
                        return Err(DAError {
                            error: anyhow!(
                                "Mismatch, blob ids len [{}] != chunk size [{}]",
                                blob_ids.len(),
                                blob.chunks
                            ),
                            is_retriable: false,
                        });
                    }

                    let mut batch_blob = vec![];

                    for blob_id in blob_ids {
                        let Some(blob) = storage
                            .get(&blob_id)
                            .map(|data| InclusionData { data: data.clone() })
                        else {
                            return Err(DAError {
                                error: anyhow!("Failed to get blob"),
                                is_retriable: false,
                            });
                        };

                        batch_blob.extend_from_slice(&blob.data);
                    }

                    batch_blob
                }
            }
            None => blob.data,
        };

        Ok(Some(InclusionData { data }))
    }

    fn clone_boxed(&self) -> Box<dyn DataAvailabilityClient> {
        Box::new(self.clone())
    }

    fn blob_size_limit(&self) -> Option<usize> {
        Some(self.blob_size_limit)
    }

    async fn ping(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // helper to create a fresh client
    fn new_client() -> InMemoryClient {
        InMemoryClient::new(1024).unwrap()
    }

    #[tokio::test]
    async fn test_dispatch_and_retrieve_blob() {
        let client = new_client();

        let data = b"hello world".to_vec();

        // Dispatch blob
        let response = client.dispatch_blob(1, data.clone()).await.unwrap();

        // Retrieve blob and verify data matches
        let inclusion = client.get_inclusion_data(&response.blob_id).await.unwrap();
        assert_eq!(inclusion, Some(InclusionData { data: data.clone() }));
    }

    #[tokio::test]
    async fn test_nonexistent_blob_returns_none() {
        let client = new_client();
        let inclusion = client.get_inclusion_data("does_not_exist").await.unwrap();
        assert!(inclusion.is_none());
    }

    #[tokio::test]
    async fn test_clone_boxed_works_independently() {
        let client = new_client();
        let boxed = client.clone_boxed();

        let data = b"clone test".to_vec();
        let resp = boxed.dispatch_blob(2, data.clone()).await.unwrap();

        // Ensure data is accessible from the original client too (shared storage)
        let inclusion = client.get_inclusion_data(&resp.blob_id).await.unwrap();
        assert_eq!(inclusion, Some(InclusionData { data: data.clone() }));
    }

    #[tokio::test]
    async fn test_ping_returns_true() {
        let client = new_client();
        assert!(client.ping().await.unwrap());
    }

    #[test]
    fn test_blob_size_limit_returns_correct_value() {
        let limit = 4096;
        let client = InMemoryClient::new(limit).unwrap();
        assert_eq!(client.blob_size_limit(), Some(limit));
    }

    #[tokio::test]
    async fn test_multiple_blobs_stored_independently() {
        let client = new_client();

        let data1 = b"first blob".to_vec();
        let data2 = b"second blob".to_vec();

        let resp1 = client.dispatch_blob(1, data1.clone()).await.unwrap();
        let resp2 = client.dispatch_blob(2, data2.clone()).await.unwrap();

        assert_ne!(resp1.blob_id, resp2.blob_id);

        let retrieved1 = client.get_inclusion_data(&resp1.blob_id).await.unwrap();
        let retrieved2 = client.get_inclusion_data(&resp2.blob_id).await.unwrap();

        assert_eq!(retrieved1, Some(InclusionData { data: data1 }));
        assert_eq!(retrieved2, Some(InclusionData { data: data2 }));
    }
}
