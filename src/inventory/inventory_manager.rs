use crate::inventory::cluster::Cluster;
use crate::inventory::deployment::Deployment;
use crate::inventory::environment::Environment;
pub(crate) use crate::inventory::server::Server;
use anyhow::{Context, Result};
use std::path::Path;
use tokio;
use tokio::io::AsyncWriteExt;

pub struct InventoryManager {
    inventory_file_name: String,
    deployment: Option<Deployment>,
}

impl InventoryManager {
    pub fn new(inventory_file_name: String) -> Self {
        Self {
            inventory_file_name,
            deployment: None,
        }
    }

    pub async fn load_inventory_from_file(&mut self) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.inventory_file_name)
            .await
            .with_context(|| {
                format!(
                    "Failed to read inventory file: {}",
                    self.inventory_file_name
                )
            })?;

        self.deployment = Some(serde_yaml::from_str(&content).with_context(|| {
            format!(
                "Failed to deserialize inventory file: {}",
                self.inventory_file_name
            )
        })?);

        if self.deployment.is_none() {
            return Err(anyhow::anyhow!(
                "Failed to load inventory: Deployment is None"
            ));
        }

        Ok(())
    }

    pub async fn save_inventory_to_file(&self, inventory_file_name: &str) -> Result<()> {
        let path = Path::new(&inventory_file_name);

        let mut output = tokio::fs::File::create(&path).await.with_context(|| {
            format!("Failed to create output inventory file: {inventory_file_name}")
        })?;

        output.write_all(b"---\n").await.with_context(|| {
            format!("Failed to write header to inventory file: {inventory_file_name}")
        })?;

        let serialized = serde_yaml::to_string(&self.deployment).with_context(|| {
            format!("Failed to serialize deployment for inventory file: {inventory_file_name}")
        })?;

        output
            .write_all(serialized.as_bytes())
            .await
            .with_context(|| {
                format!(
                    "Failed to write serialized deployment to inventory file: {inventory_file_name}"
                )
            })?;

        output.write_all(b"...").await.with_context(|| {
            format!("Failed to write footer to inventory file: {inventory_file_name}")
        })?;

        Ok(())
    }

    fn get_default_environment(&self) -> Result<&Environment> {
        if let Some(deployment) = &self.deployment {
            let default_environment = deployment
                .environments
                .iter()
                .find(|env| env.name == deployment.default_environment_name);

            Ok(default_environment.context("No default environment found")?)
        } else {
            Err(anyhow::anyhow!("No deployment loaded"))
        }
    }

    fn get_default_cluster_private<'a>(&self, environment: &'a Environment) -> Result<&'a Cluster> {
        let default_cluster = environment
            .clusters
            .iter()
            .find(|cluster| cluster.name == environment.default_cluster_name);
        default_cluster.ok_or_else(|| anyhow::anyhow!("No default cluster found"))
    }

    pub fn get_default_cluster(&self) -> Cluster {
        if let Ok(default_environment) = self.get_default_environment() {
            if let Ok(default_cluster) = self.get_default_cluster_private(default_environment) {
                let default_cluster_clone = Cluster::from(default_cluster);
                return default_cluster_clone;
            }
        }
        Cluster::new()
    }
}
