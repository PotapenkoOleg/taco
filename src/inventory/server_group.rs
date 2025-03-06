use crate::inventory::server::Server;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ServerGroup {
    pub name: String,
    pub servers: Vec<Server>,
}
