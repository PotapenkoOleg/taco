use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;
use serde::{Serialize, Deserialize};
use anyhow::{Context, Result};
use tokio;
use tokio::io::AsyncWriteExt;

pub struct InventoryManager {
    inventory_file_name: String,
    pub deployment: Option<Deployment>,
}

impl InventoryManager {
    pub fn new(inventory_file_name: &str) -> Self {
        Self { inventory_file_name: inventory_file_name.to_string(), deployment: None }
    }

    pub async fn load_inventory_from_file(&mut self) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.inventory_file_name)
            .await
            .with_context(|| format!("Failed to read inventory file: {}", self.inventory_file_name))?;
        self.deployment = Some(
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to deserialize inventory file: {}", self.inventory_file_name))?
        );
        Ok(())
    }

    pub async fn save_inventory_to_file(&self, deployment: &Option<Deployment>, file_name: Option<String>) -> Result<()> {
        let inventory_file_name = match file_name {
            Some(name) => name,
            None => self.inventory_file_name.clone(),
        };
        let path = Path::new(&inventory_file_name);
        let mut output = tokio::fs::File::create(&path)
            .await
            .with_context(|| format!("Failed to create output inventory file: {}", inventory_file_name))?;

        output.write_all(b"---\n")
            .await
            .with_context(|| format!("Failed to write header to inventory file: {}", inventory_file_name))?;

        let serialized = serde_yaml::to_string(deployment)
            .with_context(|| format!("Failed to serialize deployment for inventory file: {}", inventory_file_name))?;
        output.write_all(serialized.as_bytes())
            .await
            .with_context(|| format!("Failed to write serialized deployment to inventory file: {}", inventory_file_name))?;

        output.write_all(b"...")
            .await
            .with_context(|| format!("Failed to write tail to inventory file: {}", inventory_file_name))?;
        Ok(())
    }

    /// Returns environments matching the default environment name from the deployment.
    pub fn get_environments(&self) -> Result<Vec<&Environment>> {
        if let Some(deployment) = &self.deployment {
            let envs: Vec<&Environment> = deployment.environments.iter()
                .filter(|env| env.name == deployment.default_environment_name)
                .collect();
            if envs.is_empty() {
                Err(anyhow::anyhow!("No environment found matching default environment name: {}", deployment.default_environment_name))
            } else {
                Ok(envs)
            }
        } else {
            Err(anyhow::anyhow!("No deployment loaded"))
        }
    }

    /// Returns clusters within an environment that match the environment's default cluster name.
    pub fn get_clusters(environment: &Environment) -> Result<Vec<&Cluster>> {
        let clusters: Vec<&Cluster> = environment.clusters.iter()
            .filter(|c| c.name == environment.default_cluster_name)
            .collect();
        if clusters.is_empty() {
            Err(anyhow::anyhow!("No cluster found matching default cluster name: {}", environment.default_cluster_name))
        } else {
            Ok(clusters)
        }
    }

    /// Returns a mapping of server group names to their corresponding servers from the given cluster.
    pub fn get_server_groups(cluster: &Cluster) -> std::collections::HashMap<String, Vec<Server>> {
        cluster.server_groups.iter()
            .map(|sg| (sg.name.clone(), sg.servers.clone()))
            .collect()
    }

    /// Internal helper that collects servers from the specified server group into the given HashSet.
    fn collect_servers_in_server_group_internal(
        servers: &mut std::collections::HashSet<Server>,
        server_group_name: &str,
        cluster: &Cluster,
        server_groups: &std::collections::HashMap<String, Vec<Server>>
    ) -> Result<()> {
        if let Some(sg) = server_groups.get(server_group_name) {
            for server in sg {
                let new_server = Server::from(
                    server,
                    (&cluster.default_port,
                     &cluster.default_db_name,
                     &cluster.default_user,
                     &cluster.default_password,
                     &cluster.default_connect_timeout_sec),
                );
                servers.insert(new_server);
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Server group '{}' not found in cluster '{}'", server_group_name, cluster.name))
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
                                let _ = Self::collect_servers_in_server_group_internal(&mut servers, key, cluster, &server_groups);
                            }
                        } else {
                            let _ = Self::collect_servers_in_server_group_internal(&mut servers, server_group_name, cluster, &server_groups);
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
                    &default_cluster.default_connect_timeout_sec
                ),
            );
            servers.insert(new_server);
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Deployment {
    pub name: String,
    pub default_environment_name: String,
    pub environments: Vec<Environment>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Environment {
    pub name: String,
    pub default_cluster_name: String,
    pub clusters: Vec<Cluster>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Cluster {
    pub name: String,
    pub default_port: Option<i32>,
    pub default_db_name: Option<String>,
    pub default_user: Option<String>,
    pub default_password: Option<String>,
    pub default_connect_timeout_sec: Option<i32>,
    pub server_groups: Vec<ServerGroup>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ServerGroup {
    pub name: String,
    pub servers: Vec<Server>,
}

// https://docs.rs/postgres/latest/postgres/config/struct.Config.html#
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct Server {
    pub host: String,
    pub port: Option<i32>,
    pub db_name: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub connect_timeout_sec: Option<i32>,
}

impl Server {
    /// Creates a new Server instance, filling in any missing fields from the provided defaults.
    /// 
    /// # Arguments
    /// * `from` - The source server containing base configuration
    /// * `defaults` - A tuple containing default values for (port, db_name, user, password, connect_timeout_sec)
    pub fn from(
        from: &Server,
        defaults: (&Option<i32>, &Option<String>, &Option<String>, &Option<String>, &Option<i32>),
    ) -> Self {
        let (port, db_name, user, password, connect_timeout_sec) = defaults;
        Self {
            host: from.host.clone(),
            port: from.port.or_else(|| port.clone()),
            db_name: from.db_name.clone().or_else(|| db_name.clone()),
            user: from.user.clone().or_else(|| user.clone()),
            password: from.password.clone().or_else(|| password.clone()),
            connect_timeout_sec: from.connect_timeout_sec.or_else(|| connect_timeout_sec.clone()),
        }
    }

    /// Sets the database name for this server.
    pub fn set_db_name(&mut self, db_name: String) {
        self.db_name = Some(db_name);
    }

    /// Returns a clone of the database name if set.
    pub fn get_db_name(&self) -> Option<String> {
        self.db_name.clone()
    }
}

impl fmt::Display for Server {
    /// Formats the server as a PostgreSQL connection string.
    /// 
    /// # Panics
    /// Panics if any required field (port, db_name, user, password, connect_timeout_sec) is None.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "host={} port={} dbname={} user={} password={} connect_timeout={} application_name=taco",
            self.host,
            self.port.expect("port must be set"),
            self.db_name.as_ref().expect("db_name must be set"),
            self.user.as_ref().expect("user must be set"),
            self.password.as_ref().expect("password must be set"),
            self.connect_timeout_sec.expect("connect_timeout_sec must be set")
        )
    }
}