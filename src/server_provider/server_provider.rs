use crate::inventory::server::Server;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use std::collections::HashMap;

pub struct ServerProvider {
    server_groups: HashMap<String, Vec<Server>>,
}

impl ServerProvider {
    pub async fn new(server_groups: HashMap<String, Vec<Server>>) -> Self {
        Self { server_groups }
    }

    pub fn get_servers_in_group(&self, server_group_name: &str) -> Option<Vec<Server>> {
        match self.server_groups.get(server_group_name) {
            Some(servers) => Some(servers.clone()),
            None => None,
        }
    }

    pub fn update_server_groups(&mut self, main_server_group: Vec<Server>) {
        self.server_groups.remove(SERVER_GROUP_ALL);
        let static_server_groups: HashMap<String, Vec<String>> = self
            .server_groups
            .drain()
            .map(|(group_name, mut servers)| {
                let server_names = servers.drain(..).map(|s| s.host).collect();
                (group_name, server_names)
            })
            .collect();
        let main_server_group_map: HashMap<String, Server> = main_server_group
            .par_iter()
            .map(|s| (s.host.clone(), s.clone()))
            .collect();
        for (k, v) in static_server_groups {
            let servers: Vec<Server> = v
                .par_iter()
                .map(|s| main_server_group_map[s].clone())
                .collect();
            self.server_groups.insert(k, servers);
        }

        const SERVER_GROUP_ALL: &str = "all"; // all nodes in static config
        const SERVER_GROUP_ONLINE: &str = "online"; // is_node_online - any node online
        const SERVER_GROUP_CONS: &str = "cons"; // is_node_consistent - any consistent node
        const SERVER_GROUP_PGL: &str = "pgl"; // postgres_is_leader - postgres replication leader nodes (citus workers and coordinators)
        const SERVER_GROUP_PGR: &str = "pgr"; // postgres_is_replica - postgres replication replica nodes (citus workers and coordinators)
        const SERVER_GROUP_CLC: &str = "clc"; // citus_is_leader_coordinator_node - citus leader coordinator nodes(CITUS 13+ can have many leaders)
        const SERVER_GROUP_CRC: &str = "crc"; // citus_is_replica_coordinator_node - citus replica coordinator nodes
        const SERVER_GROUP_CLW: &str = "clw"; // citus_is_leader_worker_node - citus leader worker nodes
        const SERVER_GROUP_CRW: &str = "crw"; // citus_is_replica_worker_node - citus replica worker nodes
        const SERVER_GROUP_CAW: &str = "caw"; // citus_is_active_worker_node - citus active worker nodes(exclude nodes without shards)
        const SERVER_GROUP_PP: &str = "pp"; // patroni_is_primary - patroni primary nodes (citus workers and coordinators)
        const SERVER_GROUP_PR: &str = "pr"; // patroni_is_replica - patroni replica nodes (citus workers and coordinators)
        const SERVER_GROUP_PRW: &str = "prw"; // patroni_is_read_write - patroni read write nodes (citus workers and coordinators)
        const SERVER_GROUP_HAPROXY_RW: &str = "haproxy_rw"; // HAProxy read-write
        const SERVER_GROUP_HAPROXY_R: &str = "haproxy_r"; // HAProxy read-only

        // region SERVER_GROUP_ALL
        self.server_groups
            .insert(SERVER_GROUP_ALL.to_string(), main_server_group.clone());
        // endregion

        // region SERVER_GROUP_ONLINE
        let online_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.is_node_online.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups
            .insert(SERVER_GROUP_ONLINE.to_string(), online_server_group);
        // endregion

        // region SERVER_GROUP_CONS
        let consistent_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.is_node_consistent.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups
            .insert(SERVER_GROUP_CONS.to_string(), consistent_server_group);
        // endregion

        // region SERVER_GROUP_PGL
        let postgres_leader_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.postgres_is_leader.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups
            .insert(SERVER_GROUP_PGL.to_string(), postgres_leader_server_group);
        // endregion

        // region SERVER_GROUP_PGR
        let postgres_replica_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.postgres_is_replica.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups
            .insert(SERVER_GROUP_PGR.to_string(), postgres_replica_server_group);
        // endregion

        // region SERVER_GROUP_CLC
        let citus_leader_coordinator_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.citus_is_leader_coordinator_node.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups.insert(
            SERVER_GROUP_CLC.to_string(),
            citus_leader_coordinator_server_group,
        );
        // endregion

        // region SERVER_GROUP_CRC
        let citus_replica_coordinator_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.citus_is_replica_coordinator_node.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups.insert(
            SERVER_GROUP_CRC.to_string(),
            citus_replica_coordinator_server_group,
        );
        // endregion

        // region SERVER_GROUP_CLW
        let citus_leader_worker_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.citus_is_leader_worker_node.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups.insert(
            SERVER_GROUP_CLW.to_string(),
            citus_leader_worker_server_group,
        );
        // endregion

        // region SERVER_GROUP_CRW
        let citus_replica_worker_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.citus_is_replica_worker_node.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups.insert(
            SERVER_GROUP_CRW.to_string(),
            citus_replica_worker_server_group,
        );
        // endregion

        // region SERVER_GROUP_CAW
        let citus_active_worker_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.citus_is_active_worker_node.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups.insert(
            SERVER_GROUP_CAW.to_string(),
            citus_active_worker_server_group,
        );
        // endregion

        // region SERVER_GROUP_PP
        let patroni_primary_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.patroni_is_primary.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups
            .insert(SERVER_GROUP_PP.to_string(), patroni_primary_server_group);
        // endregion

        // region SERVER_GROUP_PR
        let patroni_replica_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.patroni_is_replica.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups
            .insert(SERVER_GROUP_PR.to_string(), patroni_replica_server_group);
        // endregion

        // region SERVER_GROUP_PRW
        let patroni_read_write_server_group: Vec<Server> = main_server_group
            .par_iter()
            .filter(|s| s.patroni_is_read_write.unwrap())
            .map(|s| s.clone())
            .collect();
        self.server_groups.insert(
            SERVER_GROUP_PRW.to_string(),
            patroni_read_write_server_group,
        );
        // endregion

        // region SERVER_GROUP_HAPROXY_RW
        // TODO: add HAProxy to static config
        self.server_groups
            .insert(SERVER_GROUP_HAPROXY_RW.to_string(), Vec::new());
        // endregion

        // region SERVER_GROUP_HAPROXY_R
        // TODO: add HAProxy to static config
        self.server_groups
            .insert(SERVER_GROUP_HAPROXY_R.to_string(), Vec::new());
        // endregion
    }
}
