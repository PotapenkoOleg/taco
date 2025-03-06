use crate::inventory::environment::Environment;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Deployment {
    pub name: String,
    pub default_environment_name: String,
    pub environments: Vec<Environment>,
}
