use crate::facts_collector::facts_collector::FactsCollector;
use crate::inventory::inventory_manager::InventoryManager;
use crate::inventory::server::Server;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::process;
use std::sync::{Arc, Mutex};

pub struct ServerProvider {
    server_groups: HashMap<String, Vec<Server>>,
    citus_db_name: Option<String>,
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

        let mut server_groups: HashMap<String, Vec<Server>> = default_cluster
            .server_groups
            .iter()
            .map(|server_group| {
                (
                    server_group.name.clone(),
                    server_group
                        .servers
                        .iter()
                        .map(|server| {
                            Server::from(
                                server,
                                (
                                    &default_cluster.default_port,
                                    &default_cluster.default_db_name,
                                    &default_cluster.default_user,
                                    &default_cluster.default_password,
                                    &default_cluster.default_connect_timeout_sec,
                                ),
                            )
                        })
                        .collect(),
                )
            })
            .collect();

        // We use HashSet here to filter out duplicates
        let all_servers: HashSet<Server> = server_groups
            .values()
            .flat_map(|servers| {
                servers.iter().map(|server| {
                    Server::from(
                        server,
                        (
                            &default_cluster.default_port,
                            &default_cluster.default_db_name,
                            &default_cluster.default_user,
                            &default_cluster.default_password,
                            &default_cluster.default_connect_timeout_sec,
                        ),
                    )
                })
            })
            .collect();

        server_groups.insert("all".to_string(), Vec::from_iter(all_servers));

        default_cluster.server_groups.clear();

        let citus_db_name = default_cluster.citus_db_name;

        Self {
            server_groups,
            citus_db_name,
        }
    }

    pub fn get_servers(&self, server_group_name: &String) -> Vec<Server> {
        self.server_groups[server_group_name].clone()
    }

    pub fn get_servers_as_ref_mut(&mut self, server_group_name: &String) -> &mut Vec<Server> {
        self.server_groups.get_mut(server_group_name).unwrap()
    }

    pub async fn collect_facts(&mut self, settings: &Arc<Mutex<HashMap<String, String>>>) {
        let all_servers = self.server_groups.get_mut("all").unwrap();
        let mut facts_collector = FactsCollector::new(all_servers, &self.citus_db_name);
        facts_collector.collect_facts(settings).await;
    }
}
