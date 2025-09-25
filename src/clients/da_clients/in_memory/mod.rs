use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use uuid::Uuid;

use crate::clients::da_clients::{
    DataAvailabilityClient,
    types::{DAError, DispatchResponse, InclusionData},
};

#[derive(Clone, Debug)]
pub struct InMemoryClient {
    storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    blob_size_limit: usize,
}

impl InMemoryClient {
    pub fn new(blob_size_limit: usize) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            blob_size_limit,
        }
    }
}

#[async_trait]
impl DataAvailabilityClient for InMemoryClient {
    async fn dispatch_blob(
        &self,
        _batch_number: u32,
        data: Vec<u8>,
    ) -> Result<DispatchResponse, DAError> {
        let blob_id = format!("inmem-{}", Uuid::new_v4());

        self.storage.lock().unwrap().insert(blob_id.clone(), data);

        Ok(DispatchResponse { blob_id })
    }

    async fn get_inclusion_data(&self, blob_id: &str) -> Result<Option<InclusionData>, DAError> {
        let storage = self.storage.lock().unwrap();
        Ok(storage
            .get(blob_id)
            .map(|data| InclusionData { data: data.clone() }))
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
