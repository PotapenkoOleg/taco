use crate::facts_collector::postgres_facts_collector::PostgresFactsCollector;
use crate::inventory::inventory_manager::Server;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct FactsCollector<'a> {
    servers: &'a mut Vec<Server>,
}

impl<'a> FactsCollector<'a> {
    pub fn new(servers: &'a mut Vec<Server>) -> Self {
        FactsCollector { servers }
    }

    pub async fn collect_facts(&mut self, settings: &Arc<Mutex<HashMap<String, String>>>) {
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
            println!("{}", server.to_string());
            if let Some(true) = collect_postgres_facts {
                println!("Collecting Postgres Facts");
                // let connection_string = &server.to_string();
                // let postgres_facts_collector = PostgresFactsCollector::new(connection_string);
                // //let pg_stat_replication_result = postgres_facts_collector.check_pg_stat_replication().await.unwrap();
                // //let pg_stat_wal_receiver_result = postgres_facts_collector.check_pg_stat_wal_receiver().await.unwrap();
            }
            if let Some(true) = collect_citus_facts {
                println!("Collecting Citus Facts");
                // let connection_string = &server.to_string();
                // let postgres_facts_collector = PostgresFactsCollector::new(connection_string);
                // //let pg_stat_replication_result = postgres_facts_collector.check_pg_stat_replication().await.unwrap();
                // //let pg_stat_wal_receiver_result = postgres_facts_collector.check_pg_stat_wal_receiver().await.unwrap();
            }
            if let Some(true) = collect_patroni_facts {
                println!("Collecting Patroni Facts");
                // let connection_string = &server.to_string();
                // let postgres_facts_collector = PostgresFactsCollector::new(connection_string);
                // //let pg_stat_replication_result = postgres_facts_collector.check_pg_stat_replication().await.unwrap();
                // //let pg_stat_wal_receiver_result = postgres_facts_collector.check_pg_stat_wal_receiver().await.unwrap();
            }
        }
    }
}
