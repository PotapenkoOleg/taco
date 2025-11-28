use crate::inventory::cluster::Cluster;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Environment {
    pub name: String,
    pub default_cluster_name: String,
    pub clusters: Vec<Cluster>,
}
