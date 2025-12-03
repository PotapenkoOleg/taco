use crate::facts_collector::citus_facts_collector::CitusFactsCollector;
use crate::facts_collector::patroni_facts_collector::PatroniFactsCollector;
use crate::facts_collector::postgres_facts_collector::PostgresFactsCollector;
use crate::inventory::inventory_manager::Server;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub struct FactsCollector<'a> {
    servers: &'a mut Vec<Server>,
    citus_db_name: &'a Option<String>,
    facts_collected: bool,
}

impl<'a> FactsCollector<'a> {
    pub fn new(servers: &'a mut Vec<Server>, citus_db_name: &'a Option<String>) -> Self {
        FactsCollector {
            servers,
            citus_db_name,
            facts_collected: false,
        }
    }

    pub async fn collect_facts(&mut self, settings: &Arc<Mutex<HashMap<String, String>>>) {
        self.facts_collected = true;

        let mut collect_postgres_facts: Option<bool> = None;
        let mut collect_citus_facts: Option<bool> = None;
        let mut collect_patroni_facts: Option<bool> = None;
        let mut check_cluster_consistency: Option<bool> = None;
        {
            // this block for mutex release
            let settings_lock = settings.lock().unwrap();
            match settings_lock.get(&"collect_postgres_facts".to_string()) {
                Some(value) => {
                    collect_postgres_facts = Some(value == "true");
                }
                _ => {}
            }
            match settings_lock.get(&"collect_citus_facts".to_string()) {
                Some(value) => {
                    collect_citus_facts = Some(value == "true");
                }
                _ => {}
            }
            match settings_lock.get(&"collect_patroni_facts".to_string()) {
                Some(value) => {
                    collect_patroni_facts = Some(value == "true");
                }
                _ => {}
            }
            match settings_lock.get(&"check_cluster_consistency".to_string()) {
                Some(value) => {
                    check_cluster_consistency = Some(value == "true");
                }
                _ => {}
            }
        }

        for server in self.servers.iter_mut() {
            let connection_string = &server.to_string();
            if let Some(true) = collect_postgres_facts {
                let postgres_facts_collector = PostgresFactsCollector::new(connection_string);
                let pg_stat_replication_result =
                    postgres_facts_collector.check_pg_stat_replication().await;
                let pg_stat_wal_receiver_result =
                    postgres_facts_collector.check_pg_stat_wal_receiver().await;
                match pg_stat_replication_result {
                    Ok(value) => {
                        if !value.is_empty() {
                            server.postgres_is_leader = Some(true);
                        } else {
                            server.postgres_is_leader = Some(false);
                        }
                        server.is_node_online = Some(true);
                    }
                    _ => {
                        server.is_node_online = Some(false);
                    }
                }
                match pg_stat_wal_receiver_result {
                    Ok(value) => {
                        if !value.is_empty() {
                            server.postgres_is_replica = Some(true);
                        } else {
                            server.postgres_is_replica = Some(false);
                        }
                    }
                    _ => {}
                }
            }

            if let Some(true) = collect_patroni_facts {
                let patroni_connection_string = format!("http://{}:8008/", server.host);
                let patroni_facts_collector =
                    PatroniFactsCollector::new(&patroni_connection_string);
                let node_status = patroni_facts_collector.check_node_status().await;
                match node_status {
                    Ok(value) => {
                        server.patroni_is_primary = value.is_primary;
                        server.patroni_is_replica = value.is_replica;
                        server.patroni_is_read_write = value.is_read_write;
                        server.patroni_is_read_only = value.is_read_only;
                    }
                    _ => {}
                }
            }
        }

        let first_server_online = self
            .servers
            .iter_mut()
            .find(|server| server.is_node_online.unwrap() == true);

        if let Some(server_online) = first_server_online {
            if let Some(true) = collect_citus_facts {
                let db_name_temp = server_online.db_name.clone();
                server_online.db_name = Some(self.citus_db_name.as_ref().unwrap().clone());
                let connection_string = &server_online.to_string();
                server_online.db_name = db_name_temp;
                let citus_facts_collector = CitusFactsCollector::new(connection_string);
                let active_worker_nodes = citus_facts_collector.get_active_worker_nodes().await;
                let x = citus_facts_collector.get_pg_dist_node_info().await;
                match active_worker_nodes {
                    Ok(value) => {
                        let active_workers: HashSet<String> = value
                            .iter()
                            .map(|v| v.node_name.as_ref().unwrap().clone())
                            .collect();
                        for server in self.servers.iter_mut() {
                            if active_workers.contains(&server.host) {
                                server.citus_is_active_worker_node = Some(true);
                            } else {
                                server.citus_is_active_worker_node = Some(false);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // TODO: check every server for consistency
        if let Some(true) = check_cluster_consistency {
            if self.servers.len() == 1 {
                return;
            }
        }
    }

    pub fn check_cluster_consistency(&mut self, settings: &Arc<Mutex<HashMap<String, String>>>) {
        if !self.facts_collected {
            println!("Facts are not collected yet. Please run collect_facts() first");
            return;
        }
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

        if let Some(true) = check_cluster_consistency {
            if self.servers.len() == 1
                && self.servers[0].postgres_is_leader == Some(false)
                && self.servers[0].postgres_is_replica == Some(false)
            {
                println!("CLUSTER IS CONSISTENT");
                return;
            }
        }
    }
}
