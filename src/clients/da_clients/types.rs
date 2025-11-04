use std::{error, fmt::Display};

use serde::{Deserialize, Serialize};

/// `DAError` is the error type returned by the DA clients.
#[derive(Debug)]
pub struct DAError {
    pub error: anyhow::Error,
    pub is_retriable: bool,
}

impl DAError {
    pub fn is_retriable(&self) -> bool {
        self.is_retriable
    }
}

impl Display for DAError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = if self.is_retriable {
            "retriable"
        } else {
            "fatal"
        };
        write!(f, "{kind} data availability client error: {}", self.error)
    }
}

impl error::Error for DAError {}

/// `DispatchResponse` is the response received from the DA layer after dispatching a blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResponse {
    /// The blob_id is needed to fetch the inclusion data.
    pub blob_id: String,
}

impl From<String> for DispatchResponse {
    fn from(blob_id: String) -> Self {
        DispatchResponse { blob_id }
    }
}

/// `InclusionData` is the data needed to verify on L1 that a blob is included in the DA layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InclusionData {
    /// The inclusion data serialized by the DA client. Serialization is done in a way that allows
    /// the deserialization of the data in Solidity contracts.
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViaDaBlob {
    pub chunks: usize,
    pub data: Vec<u8>,
}

impl ViaDaBlob {
    pub fn new(chunks: usize, data: Vec<u8>) -> Self {
        Self { chunks, data }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize ViaDaBlob")
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
}

pub fn serialize_blob_ids(hex_vec: &[String]) -> anyhow::Result<Vec<u8>> {
    let mut result = Vec::new();

    for hex_str in hex_vec {
        let bytes = hex::decode(hex_str)?;
        let len = bytes.len() as u32;

        // Write 4-byte length prefix (big-endian)
        result.extend_from_slice(&len.to_be_bytes());
        result.extend_from_slice(&bytes);
    }

    Ok(result)
}

pub fn deserialize_blob_ids(data: &[u8]) -> anyhow::Result<Vec<String>> {
    let mut pos = 0;
    let mut result = Vec::new();

    while pos < data.len() {
        // Read the 4-byte length prefix
        let len_bytes: [u8; 4] = data[pos..pos + 4].try_into()?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        pos += 4;

        // Extract the chunk
        let chunk = &data[pos..pos + len];
        pos += len;

        result.push(hex::encode(chunk));
    }

    Ok(result)
}
