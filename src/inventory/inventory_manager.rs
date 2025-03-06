use crate::inventory::cluster::Cluster;
use crate::inventory::deployment::Deployment;
use crate::inventory::environment::Environment;
pub(crate) use crate::inventory::server::Server;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio;
use tokio::io::AsyncWriteExt;

pub struct InventoryManager {
    inventory_file_name: String,
    pub deployment: Option<Deployment>,
    pub default_environment_name: Option<String>,
    pub default_cluster_name: Option<String>,
}

impl InventoryManager {
    pub fn new(inventory_file_name: &str) -> Self {
        Self {
            inventory_file_name: inventory_file_name.to_string(),
            deployment: None,
            default_environment_name: None,
            default_cluster_name: None,
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

    pub async fn save_inventory_to_file(
        &self,
        deployment: &Option<Deployment>,
        file_name: Option<String>,
    ) -> Result<()> {
        let inventory_file_name = file_name.unwrap_or_else(|| self.inventory_file_name.clone());
        let path = Path::new(&inventory_file_name);

        let mut output = tokio::fs::File::create(&path).await.with_context(|| {
            format!("Failed to create output inventory file: {inventory_file_name}")
        })?;

        output.write_all(b"---\n").await.with_context(|| {
            format!("Failed to write header to inventory file: {inventory_file_name}")
        })?;

        let serialized = serde_yaml::to_string(deployment).with_context(|| {
            format!("Failed to serialize deployment for inventory file: {inventory_file_name}")
        })?;

        output.write_all(serialized.as_bytes())
            .await
            .with_context(|| format!("Failed to write serialized deployment to inventory file: {inventory_file_name}"))?;

        output.write_all(b"...").await.with_context(|| {
            format!("Failed to write tail to inventory file: {inventory_file_name}")
        })?;

        Ok(())
    }

    /// Returns environments matching the default environment name from the deployment.
    pub fn get_environments(&self) -> Result<Vec<&Environment>> {
        if let Some(deployment) = &self.deployment {
            let envs: Vec<&Environment> = deployment
                .environments
                .iter()
                .filter(|env| env.name == deployment.default_environment_name)
                .collect();
            if envs.is_empty() {
                Err(anyhow::anyhow!(
                    "No environment found matching default environment name: {}",
                    deployment.default_environment_name
                ))
            } else {
                Ok(envs)
            }
        } else {
            Err(anyhow::anyhow!("No deployment loaded"))
        }
    }

    /// Returns clusters within an environment that match the environment's default cluster name.
    pub fn get_clusters(environment: &Environment) -> Result<Vec<&Cluster>> {
        let clusters: Vec<&Cluster> = environment
            .clusters
            .iter()
            .filter(|c| c.name == environment.default_cluster_name)
            .collect();
        if clusters.is_empty() {
            Err(anyhow::anyhow!(
                "No cluster found matching default cluster name: {}",
                environment.default_cluster_name
            ))
        } else {
            Ok(clusters)
        }
    }

    /// Returns a mapping of server group names to their corresponding servers from the given cluster.
    pub fn get_server_groups(cluster: &Cluster) -> std::collections::HashMap<String, Vec<Server>> {
        cluster
            .server_groups
            .iter()
            .map(|sg| (sg.name.clone(), sg.servers.clone()))
            .collect()
    }

    /// Internal helper that collects servers from the specified server group into the given HashSet.
    fn collect_servers_in_server_group_internal(
        servers: &mut std::collections::HashSet<Server>,
        server_group_name: &str,
        cluster: &Cluster,
        server_groups: &std::collections::HashMap<String, Vec<Server>>,
    ) -> Result<()> {
        if let Some(sg) = server_groups.get(server_group_name) {
            for server in sg {
                let new_server = Server::from(
                    server,
                    (
                        &cluster.default_port,
                        &cluster.default_db_name,
                        &cluster.default_user,
                        &cluster.default_password,
                        &cluster.default_connect_timeout_sec,
                    ),
                );
                servers.insert(new_server);
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Server group '{}' not found in cluster '{}'",
                server_group_name,
                cluster.name
            ))
        }
    }

    /// Returns the servers for the specified server group. If "all" is provided (case-insensitive),
    /// servers from every group are combined.
    pub fn get_servers(&self, server_group_name: &String) -> std::collections::HashSet<Server> {
        if let Ok(envs) = self.get_environments() {
            if let Some(environment) = envs.first() {
                if let Ok(clusters) = Self::get_clusters(environment) {
                    if let Some(cluster) = clusters.first() {
                        let server_groups = Self::get_server_groups(cluster);
                        let mut servers = std::collections::HashSet::new();
                        if server_group_name.to_lowercase().trim() == "all" {
                            for key in server_groups.keys() {
                                let _ = Self::collect_servers_in_server_group_internal(
                                    &mut servers,
                                    key,
                                    cluster,
                                    &server_groups,
                                );
                            }
                        } else {
                            let _ = Self::collect_servers_in_server_group_internal(
                                &mut servers,
                                server_group_name,
                                cluster,
                                &server_groups,
                            );
                        }
                        return servers;
                    }
                }
            }
        }
        std::collections::HashSet::new()
    }
}

fn collect_servers_in_server_group(
    servers: &mut HashSet<Server>,
    server_group_name: &String,
    default_cluster: &&Cluster,
    server_groups: &HashMap<String, Vec<Server>>,
) {
    if let Some(servers_in_group) = server_groups.get(server_group_name) {
        for server in servers_in_group {
            let new_server = Server::from(
                server,
                (
                    &default_cluster.default_port,
                    &default_cluster.default_db_name,
                    &default_cluster.default_user,
                    &default_cluster.default_password,
                    &default_cluster.default_connect_timeout_sec,
                ),
            );
            servers.insert(new_server);
        }
    }
}
