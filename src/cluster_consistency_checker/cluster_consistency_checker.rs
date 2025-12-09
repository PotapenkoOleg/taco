use crate::inventory::inventory_manager::Server;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ClusterConsistencyChecker<'a> {
    settings: &'a Arc<Mutex<HashMap<String, String>>>,
    servers: &'a mut Vec<Server>,
}
impl<'a> ClusterConsistencyChecker<'a> {
    pub fn new(
        settings: &'a Arc<Mutex<HashMap<String, String>>>,
        servers: &'a mut Vec<Server>,
    ) -> Self {
        Self { settings, servers }
    }

    pub fn check_cluster_consistency(&mut self) -> bool {
        let mut check_cluster_consistency: Option<bool> = None;
        {
            // this block for mutex release
            let settings_lock = self.settings.lock().unwrap();
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
                return true;
            }
            let mut results: Vec<bool> = Vec::new();
            for server in self.servers.iter_mut() {
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
