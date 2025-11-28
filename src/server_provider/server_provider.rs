use crate::inventory::cluster::Cluster;
use crate::inventory::inventory_manager::InventoryManager;
use crate::inventory::server::Server;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::process;

pub struct ServerProvider {
    default_cluster: Cluster,
    server_groups: HashMap<String, Vec<Server>>
}

impl ServerProvider {
    pub async fn new(inventory_file_name: String) -> Self {
        let mut inventory_manager = InventoryManager::new(inventory_file_name);
        let result = inventory_manager.load_inventory_from_file().await;
        if result.is_err() {
            eprintln!("{}", result.err().unwrap().to_string().red());
            process::exit(1);
        }
        let mut default_cluster = inventory_manager.get_default_cluster();

        let server_groups: HashMap<String, Vec<Server>> = default_cluster
            .server_groups
            .iter()
            .map(|server_group| (server_group.name.clone(), server_group.servers.clone()))
            .collect();
        
        default_cluster.server_groups.clear();
        
        Self {
            default_cluster,
            server_groups
        }
    }
    
    pub fn get_servers(&self, server_group_name: &String) -> HashSet<Server> {
        let mut servers = HashSet::new();
        if server_group_name.to_lowercase().trim() == "all" {
            servers = self
                .server_groups
                .values()
                .flat_map(|servers| {
                    servers.iter().map(|server| {
                        Server::from(
                            server,
                            (
                                &self.default_cluster.default_port,
                                &self.default_cluster.default_db_name,
                                &self.default_cluster.default_user,
                                &self.default_cluster.default_password,
                                &self.default_cluster.default_connect_timeout_sec,
                            ),
                        )
                    })
                })
                .collect();
        } else {
            servers = self.server_groups[server_group_name]
                .iter()
                .map(|server| {
                    Server::from(
                        server,
                        (
                            &self.default_cluster.default_port,
                            &self.default_cluster.default_db_name,
                            &self.default_cluster.default_user,
                            &self.default_cluster.default_password,
                            &self.default_cluster.default_connect_timeout_sec,
                        ),
                    )
                })
                .collect()
        }
        servers
    }
}
