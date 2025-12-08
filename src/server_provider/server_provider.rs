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

    pub async fn collect_facts(&mut self, settings: &Arc<Mutex<HashMap<String, String>>>) {
        let all_servers = self.server_groups.get_mut("all").unwrap();
        let mut facts_collector = FactsCollector::new(all_servers, &self.citus_db_name);
        facts_collector.collect_facts(settings).await;
    }

    pub fn check_cluster_consistency(
        &mut self,
        settings: &Arc<Mutex<HashMap<String, String>>>,
    ) -> bool {
        let mut check_cluster_consistency: Option<bool> = None;
        {
            // this block for mutex release
            let settings_lock = settings.lock().unwrap();
            match settings_lock.get(&"check_cluster_consistency".to_string()) {
                Some(value) => {
                    check_cluster_consistency = Some(value == "true");
                }
                _ => {}
            }
        }

        let all_servers = self.server_groups.get_mut("all").unwrap();

        if let Some(true) = check_cluster_consistency {
            if all_servers.len() == 1
                && all_servers[0].postgres_is_leader == Some(false)
                && all_servers[0].postgres_is_replica == Some(false)
            {
                return true;
            }
            let mut results: Vec<bool> = Vec::new();
            for server in all_servers {
                let result = Self::check_server_consistency(server);
                results.push(result);
            }
            return results.iter().all(|x| *x == true);
        }
        false
    }

    fn check_server_consistency(server: &mut Server) -> bool {
        if !server.is_node_online.unwrap() {
            server.is_node_consistent = Some(false);
            return false;
        }

        let mut server_flags: u16 = 0;
        // 1
        if Some(true) == server.postgres_is_leader {
            server_flags |= 1;
        }
        server_flags <<= 1;
        //2
        if Some(true) == server.postgres_is_replica {
            server_flags |= 1;
        }
        server_flags <<= 1;
        //3
        if Some(true) == server.citus_is_leader_coordinator_node {
            server_flags |= 1;
        }
        server_flags <<= 1;
        //4
        if Some(true) == server.citus_is_replica_coordinator_node {
            server_flags |= 1;
        }
        server_flags <<= 1;
        //5
        if Some(true) == server.citus_is_leader_worker_node {
            server_flags |= 1;
        }
        server_flags <<= 1;
        //6
        if Some(true) == server.citus_is_replica_worker_node {
            server_flags |= 1;
        }
        server_flags <<= 1;
        //7
        if Some(true) == server.citus_is_active_worker_node {
            server_flags |= 1;
        }
        server_flags <<= 1;
        //8
        if Some(true) == server.patroni_is_primary {
            server_flags |= 1;
        }
        server_flags <<= 1;
        // 9
        if Some(true) == server.patroni_is_replica {
            server_flags |= 1;
        }

        let leader_coordinator: u16 = 0b0000000101000010;
        let replica_coordinator: u16 = 0b0000000010100001;
        let leader_worker: u16 = 0b0000000100010110;
        let replica_worker: u16 = 0b0000000010001001;

        if server_flags & leader_coordinator == leader_coordinator {
            server.is_node_consistent = Some(true);
            return true;
        }
        if server_flags & replica_coordinator == replica_coordinator {
            server.is_node_consistent = Some(true);
            return true;
        }
        if server_flags & leader_worker == leader_worker {
            server.is_node_consistent = Some(true);
            return true;
        }
        if server_flags & replica_worker == replica_worker {
            server.is_node_consistent = Some(true);
            return true;
        }

        false
    }
}
