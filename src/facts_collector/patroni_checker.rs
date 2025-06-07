/// This module provides functionality to query Patroni health check endpoints.
/// 
/// The Patroni REST API provides several health check endpoints that can be used to monitor
/// the health of a Patroni cluster. This module provides a simple interface to query these
/// endpoints and get the status of a Patroni node.
/// 
/// # Examples
/// 
/// ```
/// use taco::facts_collector::patroni_checker::PatroniChecker;
/// use taco::inventory::server::Server;
/// 
/// async fn check_patroni_health(server: &Server) -> anyhow::Result<bool> {
///     let checker = PatroniChecker::new();
///     checker.check_health(server).await
/// }
/// ```
use anyhow::Result;
use reqwest::StatusCode;
use serde_json::Value;
use crate::inventory::server::Server;

pub struct PatroniChecker {
    client: reqwest::Client,
    port: u16,
}

impl PatroniChecker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            port: 8008,
        }
    }

    pub fn with_port(port: u16) -> Self {
        Self {
            client: reqwest::Client::new(),
            port,
        }
    }

    /// Constructs the base URL for Patroni API
    fn get_base_url(&self, server: &Server) -> String {
        format!("http://{}:{}", server.host, self.port)
    }

    /// Checks the health endpoint
    /// GET /health
    /// Returns HTTP status code 200 if Patroni is running, 503 if Patroni is not running
    pub async fn check_health(&self, server: &Server) -> Result<bool> {
        let url = format!("{}/health", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the cluster endpoint
    /// GET /cluster
    /// Returns information about the cluster
    pub async fn get_cluster_info(&self, server: &Server) -> Result<Value> {
        let url = format!("{}/cluster", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        let info = response.json::<Value>().await?;
        Ok(info)
    }

    /// Checks the primary endpoint
    /// GET /primary
    /// Returns HTTP status code 200 if the node is the primary, 503 otherwise
    pub async fn is_primary(&self, server: &Server) -> Result<bool> {
        let url = format!("{}/primary", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the replica endpoint
    /// GET /replica
    /// Returns HTTP status code 200 if the node is a healthy replica, 503 otherwise
    pub async fn is_replica(&self, server: &Server) -> Result<bool> {
        let url = format!("{}/replica", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the replica endpoint with lag parameter
    /// GET /replica?lag=<lag>
    /// Returns HTTP status code 200 if the node is a healthy replica and the lag is less than <lag>, 503 otherwise
    pub async fn check_replica_lag(&self, server: &Server, max_lag: &str) -> Result<bool> {
        let url = format!("{}/replica?lag={}", self.get_base_url(server), max_lag);
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the read-write endpoint
    /// GET /read-write
    /// Returns HTTP status code 200 if the node is the primary, 503 otherwise
    pub async fn is_read_write(&self, server: &Server) -> Result<bool> {
        let url = format!("{}/read-write", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the read-only endpoint
    /// GET /read-only
    /// Returns HTTP status code 200 if the node is a healthy replica, 503 otherwise
    pub async fn is_read_only(&self, server: &Server) -> Result<bool> {
        let url = format!("{}/read-only", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the standby leader endpoint
    /// GET /standby-leader
    /// Returns HTTP status code 200 if the node is the standby leader, 503 otherwise
    pub async fn is_standby_leader(&self, server: &Server) -> Result<bool> {
        let url = format!("{}/standby-leader", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the synchronous endpoint
    /// GET /synchronous
    /// Returns HTTP status code 200 if the node is a synchronous standby, 503 otherwise
    pub async fn is_sync_standby(&self, server: &Server) -> Result<bool> {
        let url = format!("{}/synchronous", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }

    /// Checks the asynchronous endpoint
    /// GET /asynchronous
    /// Returns HTTP status code 200 if the node is an asynchronous standby, 503 otherwise
    pub async fn is_async_standby(&self, server: &Server) -> Result<bool> {
        let url = format!("{}/asynchronous", self.get_base_url(server));
        let response = self.client.get(&url).send().await?;
        Ok(response.status() == StatusCode::OK)
    }
}

