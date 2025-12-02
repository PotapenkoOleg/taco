use crate::facts_collector::citus_facts_collector::CitusFactsCollector;
use crate::facts_collector::patroni_facts_collector::PatroniFactsCollector;
use crate::facts_collector::postgres_facts_collector::PostgresFactsCollector;
use crate::inventory::inventory_manager::Server;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct FactsCollector<'a> {
    servers: &'a mut Vec<Server>,
    facts_collected: bool,
}

impl<'a> FactsCollector<'a> {
    pub fn new(servers: &'a mut Vec<Server>) -> Self {
        FactsCollector {
            servers,
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

        if let Some(server) = self.servers.first() {
            if let Some(true) = collect_citus_facts {
                // TODO: set DB to Citus DB
                let connection_string = &server.to_string();
                // TODO: Restore DB
                let citus_facts_collector = CitusFactsCollector::new(connection_string);
                let active_worker_nodes = citus_facts_collector.get_active_worker_nodes().await;
                match active_worker_nodes {
                    Ok(value) => {}
                    _ => {}
                }
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
                    }
                    _ => {}
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
