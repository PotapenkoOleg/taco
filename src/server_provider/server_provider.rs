use crate::inventory::server::Server;
use std::collections::HashMap;

pub struct ServerProvider {
    server_groups: HashMap<String, Vec<Server>>,
}

impl ServerProvider {
    pub async fn new(server_groups: HashMap<String, Vec<Server>>) -> Self {
        Self { server_groups }
    }

    pub fn get_servers_in_group(&self, server_group_name: &str) -> Vec<Server> {
        self.server_groups[server_group_name].clone()
    }

    pub fn update_server_group(&mut self, server_group_name: String, server_group: Vec<Server>) {
        self.server_groups.insert(server_group_name, server_group);
    }
}
