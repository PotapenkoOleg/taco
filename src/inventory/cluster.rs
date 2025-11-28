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

impl Cluster {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            default_port: None,
            default_db_name: None,
            default_user: None,
            default_password: None,
            default_connect_timeout_sec: None,
            server_groups: Vec::new(),
        }
    }

    pub fn from(other: &Cluster) -> Self {
        Self {
            name: other.name.clone(),
            default_port: other.default_port,
            default_db_name: other.default_db_name.clone(),
            default_user: other.default_user.clone(),
            default_password: other.default_password.clone(),
            default_connect_timeout_sec: other.default_connect_timeout_sec,
            server_groups: other.server_groups.clone(),
        }
    }
}
