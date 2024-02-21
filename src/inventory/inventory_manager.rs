use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};

pub struct InventoryManager {
    inventory_file_name: String,
    deployment: Option<Deployment>,
}

impl InventoryManager {
    pub fn new(inventory_file_name: &str) -> Self {
        Self { inventory_file_name: inventory_file_name.to_string(), deployment: None }
    }

    pub fn load_inventory_from_file(&mut self) -> Result<(), serde_yaml::Error> {
        let path = Path::new(&self.inventory_file_name);
        let mut input = File::open(&path).expect("");
        let mut content = String::new();
        input.read_to_string(&mut content).expect("");
        self.deployment = Some(serde_yaml::from_str(&content)?);
        Ok(())
    }

    pub fn save_inventory_to_file(&self, deployment: Deployment) -> Result<(), serde_yaml::Error> {
        let path = Path::new(&self.inventory_file_name);
        let mut output = File::create(&path).expect("");
        output.write("---\n".as_bytes()).expect("TODO: panic message");
        let payload_str = serde_yaml::to_string(&deployment)?;
        output.write_all(payload_str.as_bytes()).expect("");
        output.write("...".as_bytes()).expect("TODO: panic message");
        Ok(())
    }

    pub fn get_connection_strings(&self, server_group_name: &String) -> Vec<String> {
        match &self.deployment {
            Some(deployment) => {
                let environments: Vec<&Environment> = deployment.environments.iter()
                    .filter(|environment| (*environment.name).cmp(&deployment.default_environment_name) == Ordering::Equal)
                    .collect();
                let default_environment = environments.first().unwrap();
                let clusters: Vec<&Cluster> = default_environment.clusters.iter()
                    .filter(|cluster| (*cluster.name).cmp(&default_environment.default_cluster_name) == Ordering::Equal)
                    .collect();
                let default_cluster = clusters.first().unwrap();
                let server_groups: HashMap<String, Vec<Server>> = default_cluster.server_groups.iter()
                    .map(|sg| (sg.name.clone(), sg.servers.clone()))
                    .collect();
                // TODO: add "all"
                //let p = server_groups.keys();
                let connection_strings = server_groups.get(server_group_name).unwrap().iter()
                    .map(|server: &Server| server.to_string())
                    .collect();

                return connection_strings;
            }
            None => {
                return Vec::new();
            }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Deployment {
    name: String,
    default_environment_name: String,
    environments: Vec<crate::inventory::inventory_manager::Environment>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Environment {
    name: String,
    default_cluster_name: String,
    clusters: Vec<Cluster>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Cluster {
    name: String,
    default_port: Option<i32>,
    default_db_name: Option<String>,
    default_user: Option<String>,
    default_password: Option<String>,
    default_connect_timeout_sec: Option<i32>,
    server_groups: Vec<ServerGroup>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ServerGroup {
    name: String,
    servers: Vec<Server>,
}

// https://docs.rs/postgres/latest/postgres/config/struct.Config.html#
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
struct Server {
    host: String,
    port: Option<i32>,
    db_name: Option<String>,
    user: Option<String>,
    password: Option<String>,
    connect_timeout_sec: Option<i32>,
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let connection_string = format!("host={} port={} dbname={} user={} password={} connect_timeout={} application_name=taco",
                                        self.host,
                                        self.port.unwrap(),
                                        self.db_name.as_ref().unwrap(),
                                        self.user.as_ref().unwrap(),
                                        self.password.as_ref().unwrap(),
                                        self.connect_timeout_sec.unwrap()
        );
        f.write_str(connection_string.as_ref()).expect("TODO: panic message");
        Ok(())
    }
}