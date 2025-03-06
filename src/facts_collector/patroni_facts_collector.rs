use anyhow::Result;
use reqwest::StatusCode;
use serde_json::Value;

pub struct PatroniFactsCollector {
    client: reqwest::Client,
}

impl PatroniFactsCollector {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn check_endpoint(&self, url: &str) -> Result<bool> {
        let response = self.client.get(url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    pub async fn is_primary(&self, base_url: &str) -> Result<bool> {
        self.check_endpoint(&format!("{}/primary", base_url)).await
    }

    pub async fn is_replica(&self, base_url: &str) -> Result<bool> {
        self.check_endpoint(&format!("{}/replica", base_url)).await
    }

    pub async fn check_replica_lag(&self, base_url: &str, max_lag: &str) -> Result<bool> {
        self.check_endpoint(&format!("{}/replica?lag={}", base_url, max_lag))
            .await
    }

    pub async fn is_healthy(&self, base_url: &str) -> Result<bool> {
        self.check_endpoint(&format!("{}/health", base_url)).await
    }

    pub async fn get_node_status(&self, base_url: &str) -> Result<Value> {
        let response = self.client.get(base_url).send().await?;
        let status = response.json::<Value>().await?;
        Ok(status)
    }

    pub async fn is_sync_standby(&self, base_url: &str) -> Result<bool> {
        self.check_endpoint(&format!("{}/synchronous", base_url)).await
    }

    pub async fn is_async_standby(&self, base_url: &str) -> Result<bool> {
        self.check_endpoint(&format!("{}/asynchronous", base_url)).await
    }

    pub async fn check_liveness(&self, base_url: &str) -> Result<bool> {
        self.check_endpoint(&format!("{}/liveness", base_url)).await
    }

    pub async fn check_readiness(&self, base_url: &str) -> Result<bool> {
        self.check_endpoint(&format!("{}/readiness", base_url)).await
    }
}