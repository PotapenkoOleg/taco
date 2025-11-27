use anyhow::Result;
use reqwest::StatusCode;
use serde_json::Value;

pub struct PatroniFactsCollectorResult {
    healthy: Option<bool>,
    is_primary: Option<bool>,
    is_replica: Option<bool>,
    replica_has_no_lag: Option<bool>,
    is_read_write: Option<bool>,
    is_read_only: Option<bool>,
    is_standby_leader: Option<bool>,
    is_sync_standby: Option<bool>,
    is_async_standby: Option<bool>,
}

pub struct PatroniFactsCollector<'a> {
    client: reqwest::Client,
    base_url: &'a str,
}

impl<'a> PatroniFactsCollector<'a> {
    pub fn new(base_url: &'a str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// Checks the cluster endpoint
    /// GET /cluster
    /// Returns information about the cluster
    pub async fn get_cluster_info(&self) -> Result<Value> {
        let url = format!("{}/cluster", self.base_url);
        let response = self.client.get(&url).send().await?;
        let info = response.json::<Value>().await?;
        Ok(info)
    }

    /// Checks the health endpoint
    /// GET /health
    /// Returns HTTP status code 200 if Patroni is running, 503 if Patroni is not running
    pub async fn check_health(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the primary endpoint
    /// GET /primary
    /// Returns HTTP status code 200 if the node is the primary, 503 otherwise
    pub async fn is_primary(&self) -> Result<bool> {
        let url = format!("{}/primary", self.base_url);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the replica endpoint
    /// GET /replica
    /// Returns HTTP status code 200 if the node is a healthy replica, 503 otherwise
    pub async fn is_replica(&self) -> Result<bool> {
        let url = format!("{}/replica", self.base_url);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the replica endpoint with lag parameter
    /// GET /replica?lag=<lag>
    /// Returns HTTP status code 200 if the node is a healthy replica and the lag is less than <lag>, 503 otherwise
    pub async fn check_replica_lag(&self, max_lag: &str) -> Result<bool> {
        let url = format!("{}/replica?lag={}", self.base_url, max_lag);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the read-write endpoint
    /// GET /read-write
    /// Returns HTTP status code 200 if the node is the primary, 503 otherwise
    pub async fn is_read_write(&self) -> Result<bool> {
        let url = format!("{}/read-write", self.base_url);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the read-only endpoint
    /// GET /read-only
    /// Returns HTTP status code 200 if the node is a healthy replica, 503 otherwise
    pub async fn is_read_only(&self) -> Result<bool> {
        let url = format!("{}/read-only", self.base_url);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the standby leader endpoint
    /// GET /standby-leader
    /// Returns HTTP status code 200 if the node is the standby leader, 503 otherwise
    pub async fn is_standby_leader(&self) -> Result<bool> {
        let url = format!("{}/standby-leader", self.base_url);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the synchronous endpoint
    /// GET /synchronous
    /// Returns HTTP status code 200 if the node is a synchronous standby, 503 otherwise
    pub async fn is_sync_standby(&self) -> Result<bool> {
        let url = format!("{}/synchronous", self.base_url);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the asynchronous endpoint
    /// GET /asynchronous
    /// Returns HTTP status code 200 if the node is an asynchronous standby, 503 otherwise
    pub async fn is_async_standby(&self) -> Result<bool> {
        let url = format!("{}/asynchronous", self.base_url);
        let response = self.client.head(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    pub async fn check_node_status(&self) -> Result<PatroniFactsCollectorResult> {
        let result = PatroniFactsCollectorResult {
            healthy: Some(*(&self.check_health().await?)),
            is_primary: Some(*(&self.is_primary().await?)),
            is_replica: Some(*(&self.is_replica().await?)),
            replica_has_no_lag: Some(*(&self.check_replica_lag("1MB").await?)),
            is_read_write: Some(*(&self.is_read_write().await?)),
            is_read_only: Some(*(&self.is_read_only().await?)),
            is_standby_leader: Some(*(&self.is_standby_leader().await?)),
            is_sync_standby: Some(*(&self.is_sync_standby().await?)),
            is_async_standby: Some(*(&self.is_async_standby().await?)),
        };
        Ok(result)
    }
}
