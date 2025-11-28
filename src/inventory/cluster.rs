use crate::inventory::server_group::ServerGroup;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Cluster {
    pub name: String,
    pub default_port: Option<i32>,
    pub default_db_name: Option<String>,
    pub default_user: Option<String>,
    pub default_password: Option<String>,
    pub default_connect_timeout_sec: Option<i32>,
    pub server_groups: Vec<ServerGroup>,
}
