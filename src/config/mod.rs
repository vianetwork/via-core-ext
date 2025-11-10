use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DaBackend {
    Celestia,
    InMemory,
}

impl Default for DaBackend {
    fn default() -> Self {
        DaBackend::InMemory
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    /// The app port
    pub port: u16,

    /// The app address
    pub app_address: String,

    /// Metric PORT
    pub metrics_port: u16,

    /// The metrics address
    pub metrics_address: String,

    /// The DA backend
    pub da_backend: DaBackend,

    /// The DA client node url
    pub da_node_url: Option<String>,

    /// The DA client auth token
    pub da_auth_token: Option<String>,

    /// The DA blob size limit
    pub da_blob_size_limit: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = env::var("PORT")?.parse::<u16>()?;
        let metrics_port = env::var("METRICS_PORT")?.parse::<u16>()?;
        let app_address = format!("0.0.0.0:{}", port);
        let metrics_address = format!("0.0.0.0:{}", metrics_port);

        // Backend selection with safe default
        let da_backend = match env::var("VIA_DA_CLIENT_DA_BACKEND")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "celestia" => DaBackend::Celestia,
            "inmemory" | "" => DaBackend::InMemory,
            other => anyhow::bail!("Invalid DA_BACKEND value: {}", other),
        };

        tracing::info!("Start with DA backend {:?}", da_backend);

        let da_node_url = env::var("VIA_DA_CLIENT_API_NODE_URL").ok();
        let da_auth_token = env::var("VIA_DA_CLIENT_AUTH_TOKEN").ok();

        // Parse blob size limit safely, default to 1 MB if not set
        let da_blob_size_limit = env::var("VIA_DA_CLIENT_BLOB_SIZE_LIMIT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1024 * 1024);

        // Validate required Celestia settings
        if da_backend == DaBackend::Celestia {
            if da_node_url.is_none() {
                anyhow::bail!("DA_NODE_URL is required for Celestia backend");
            }
            if da_auth_token.is_none() {
                anyhow::bail!("DA_NODE_AUTH_TOKEN is required for Celestia backend");
            }
        }

        Ok(Config {
            port,
            app_address,
            metrics_port,
            metrics_address,
            da_backend,
            da_node_url,
            da_auth_token,
            da_blob_size_limit,
        })
    }
}
