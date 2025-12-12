use crate::facts_collector::citus_facts_collector::CitusFactsCollector;
use crate::facts_collector::patroni_facts_collector::PatroniFactsCollector;
use crate::facts_collector::postgres_facts_collector::PostgresFactsCollector;
use crate::inventory::inventory_manager::Server;
use crate::shared::pg_dist_node_info_result::PgDistNodeInfoResult;
use rayon::iter::ParallelIterator;
use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::task::JoinSet;

pub struct FactsCollector<'a> {
    settings: &'a Arc<Mutex<HashMap<String, String>>>,
}

impl<'a> FactsCollector<'a> {
    pub fn new(settings: &'a Arc<Mutex<HashMap<String, String>>>) -> Self {
        FactsCollector { settings }
    }

    pub async fn collect_facts(&self, servers: &mut Vec<Server>, citus_db_name: Option<String>) {
        let mut collect_patroni_facts: Option<bool> = None;
        let mut collect_citus_facts: Option<bool> = None;
        {
            // this block for mutex release
            let settings_lock = self.settings.lock().unwrap();
            match settings_lock.get(&"collect_patroni_facts".to_string()) {
                Some(value) => {
                    collect_patroni_facts = Some(value == "true");
                }
                _ => {}
            }
            match settings_lock.get(&"collect_citus_facts".to_string()) {
                Some(value) => {
                    collect_citus_facts = Some(value == "true");
                }
                _ => {}
            }
        }

        let mut join_set_extract = JoinSet::new();

        for server in servers.iter_mut() {
            let mut server_clone = server.clone();
            join_set_extract.spawn(async move {
                Self::update_postgres_status(&mut server_clone).await;
                if let Some(true) = collect_patroni_facts {
                    Self::update_patroni_status(&mut server_clone).await;
                }
                server_clone
            });
        }

        let cloned_servers = join_set_extract.join_all().await;
        let cloned_servers_dict: HashMap<String, Server> = cloned_servers
            .par_iter()
            .map(|server| (server.host.clone(), server.clone()))
            .collect();
        drop(cloned_servers);

        servers.par_iter_mut().for_each(|server| {
            let cloned_server = &cloned_servers_dict[&server.host];
            server.is_node_online = cloned_server.is_node_online;
            server.postgres_is_leader = cloned_server.postgres_is_leader;
            server.postgres_is_replica = cloned_server.postgres_is_replica;
            server.patroni_is_primary = cloned_server.patroni_is_primary;
            server.patroni_is_replica = cloned_server.patroni_is_replica;
            server.patroni_is_read_write = cloned_server.patroni_is_read_write;
            server.patroni_is_read_only = cloned_server.patroni_is_read_only;
        });
        drop(cloned_servers_dict);

        let first_server_online = servers
            .par_iter_mut()
            .find_any(|server| server.is_node_online.unwrap() == true);

        if let Some(server_online) = first_server_online {
            if let Some(true) = collect_citus_facts {
                let db_name_temp = server_online.db_name.clone();
                server_online.db_name = Some(citus_db_name.unwrap());
                let connection_string = &server_online.to_string();
                server_online.db_name = db_name_temp;
                let citus_facts_collector = CitusFactsCollector::new(connection_string);
                let active_worker_nodes = citus_facts_collector.get_active_worker_nodes().await;
                match active_worker_nodes {
                    Ok(value) => {
                        let active_workers: HashSet<String> = value
                            .par_iter()
                            .map(|v| v.node_name.as_ref().unwrap().clone())
                            .collect();
                        for server in servers.iter_mut() {
                            if active_workers.contains(&server.host) {
                                server.citus_is_active_worker_node = Some(true);
                            } else {
                                server.citus_is_active_worker_node = Some(false);
                            }
                        }
                    }
                    _ => {}
                }
                let pg_dist_node_info = citus_facts_collector.get_pg_dist_node_info().await;
                match pg_dist_node_info {
                    Ok(value) => {
                        let node_info: HashMap<String, PgDistNodeInfoResult> = value
                            .par_iter()
                            .map(|v| (v.nodename.as_ref().unwrap().clone(), (*v).clone()))
                            .collect();
                        for server in servers.iter_mut() {
                            Self::update_citus_status(server, &node_info)
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    async fn update_postgres_status(server_clone: &mut Server) {
        let postgres_connection_string = server_clone.to_string();
        let postgres_facts_collector = PostgresFactsCollector::new(&postgres_connection_string);
        let pg_stat_replication_result = postgres_facts_collector.check_pg_stat_replication().await;
        let pg_stat_wal_receiver_result =
            postgres_facts_collector.check_pg_stat_wal_receiver().await;
        match pg_stat_replication_result {
            Ok(value) => {
                if !value.is_empty() {
                    server_clone.postgres_is_leader = Some(true);
                } else {
                    server_clone.postgres_is_leader = Some(false);
                }
                server_clone.is_node_online = Some(true);
            }
            _ => {
                server_clone.is_node_online = Some(false);
            }
        }
        match pg_stat_wal_receiver_result {
            Ok(value) => {
                if !value.is_empty() {
                    server_clone.postgres_is_replica = Some(true);
                } else {
                    server_clone.postgres_is_replica = Some(false);
                }
            }
            _ => {}
        }
    }

    async fn update_patroni_status(server_clone: &mut Server) {
        let patroni_connection_string = format!("http://{}:8008/", server_clone.host);
        let patroni_facts_collector = PatroniFactsCollector::new(&patroni_connection_string);
        let node_status = patroni_facts_collector.check_node_status().await;
        match node_status {
            Ok(value) => {
                server_clone.patroni_is_primary = value.is_primary;
                server_clone.patroni_is_replica = value.is_replica;
                server_clone.patroni_is_read_write = value.is_read_write;
                server_clone.patroni_is_read_only = value.is_read_only;
            }
            _ => {}
        }
    }

    fn update_citus_status(server: &mut Server, node_info: &HashMap<String, PgDistNodeInfoResult>) {
        if let Some(node_info) = node_info.get(&server.host) {
            if server.host == node_info.nodename.clone().unwrap() {
                if let Some(groupid) = node_info.groupid
                    && let Some(noderole) = node_info.noderole.clone()
                {
                    server.citus_group_id = Some(groupid);
                    if groupid == 0 && noderole == "primary" {
                        server.citus_is_leader_coordinator_node = Some(true);
                        server.citus_is_replica_coordinator_node = Some(false);
                        server.citus_is_leader_worker_node = Some(false);
                        server.citus_is_replica_worker_node = Some(false);
                        return;
                    }
                    if groupid == 0 && noderole == "secondary" {
                        server.citus_is_leader_coordinator_node = Some(false);
                        server.citus_is_replica_coordinator_node = Some(true);
                        server.citus_is_leader_worker_node = Some(false);
                        server.citus_is_replica_worker_node = Some(false);
                        return;
                    }
                    if groupid != 0 && noderole == "primary" {
                        server.citus_is_leader_coordinator_node = Some(false);
                        server.citus_is_replica_coordinator_node = Some(false);
                        server.citus_is_leader_worker_node = Some(true);
                        server.citus_is_replica_worker_node = Some(false);
                        return;
                    }
                    if groupid != 0 && noderole == "secondary" {
                        server.citus_is_leader_coordinator_node = Some(false);
                        server.citus_is_replica_coordinator_node = Some(false);
                        server.citus_is_leader_worker_node = Some(false);
                        server.citus_is_replica_worker_node = Some(true);
                        return;
                    }
                }
            }
        }
    }
}

impl Drop for FactsCollector<'_> {
    fn drop(&mut self) {
        // println!("Dropping FactsCollector!");
    }
}
