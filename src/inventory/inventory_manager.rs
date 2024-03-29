use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::{fmt, process};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};

pub struct InventoryManager {
    inventory_file_name: String,
    pub deployment: Option<Deployment>,
}

impl InventoryManager {
    pub fn new(inventory_file_name: &str) -> Self {
        Self { inventory_file_name: inventory_file_name.to_string(), deployment: None }
    }

    pub fn load_inventory_from_file(&mut self) {
        let path = Path::new(&self.inventory_file_name);
        let input_result = File::open(&path);
        if input_result.as_ref().is_err() {
            println!("ERROR OPENING INVENTORY FILE");
            process::exit(1);
        }
        let mut content = String::new();
        let mut input = input_result.unwrap();
        let read_to_string_result = input.read_to_string(&mut content);
        if read_to_string_result.as_ref().is_err() {
            println!("ERROR READING INVENTORY FILE");
            process::exit(1);
        }
        let deserialize_result = serde_yaml::from_str(&content);
        if deserialize_result.as_ref().is_err() {
            println!("ERROR DESERIALIZING INVENTORY FILE");
            process::exit(1);
        }
        self.deployment = Some(deserialize_result.unwrap());
    }

    pub fn _save_inventory_to_file(&self, deployment: &Option<Deployment>, file_name: Option<String>) {
        let inventory_file_name = match file_name {
            Some(name) => { name }
            None => { self.inventory_file_name.clone() }
        };
        let path = Path::new(&inventory_file_name);
        let output_result = File::create(&path);
        if output_result.as_ref().is_err() {
            println!("\nERROR OPENING OUTPUT INVENTORY FILE");
            process::exit(1);
        }
        let mut output = output_result.unwrap();
        let write_header_result = output.write("---\n".as_bytes());
        if write_header_result.as_ref().is_err() {
            println!("\nERROR WRITING OUTPUT INVENTORY FILE");
            process::exit(1);
        }

        let serialize_result = serde_yaml::to_string(&deployment);
        if serialize_result.as_ref().is_err() {
            println!("\nERROR SERIALIZING OUTPUT INVENTORY FILE");
            process::exit(1);
        }
        let write_serialized_result = output.write_all(serialize_result.unwrap().as_bytes());
        if write_serialized_result.as_ref().is_err() {
            println!("\nERROR WRITING SERIALIZED INVENTORY FILE");
            process::exit(1);
        }
        let write_tail_result = output.write("...".as_bytes());
        if write_tail_result.as_ref().is_err() {
            println!("\nERROR WRITING INVENTORY FILE TAIL");
            process::exit(1);
        }
    }

    pub fn get_servers(&self, server_group_name: &String) -> HashSet<Server> {
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

                let mut servers: HashSet<Server> = HashSet::new();

                if server_group_name.to_lowercase().trim().cmp(&"all".to_string()).is_eq() {
                    for server_groups_key in server_groups.keys() {
                        collect_servers_in_server_group(&mut servers, server_groups_key, default_cluster, &server_groups);
                    }
                } else {
                    collect_servers_in_server_group(&mut servers, server_group_name, default_cluster, &server_groups);
                }

                return servers;
            }
            None => {
                return HashSet::new();
            }
        }
    }
}

fn collect_servers_in_server_group(
    servers: &mut HashSet<Server>,
    server_group_name: &String,
    default_cluster: &&Cluster,
    server_groups: &HashMap<String, Vec<Server>>,
) {
    for server in server_groups.get(server_group_name).unwrap() {
        let new_server = Server::from(
            server,
            &default_cluster.default_port,
            &default_cluster.default_db_name,
            &default_cluster.default_user,
            &default_cluster.default_password,
            &default_cluster.default_connect_timeout_sec,
        );
        servers.insert(new_server);
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Deployment {
    pub name: String,
    pub default_environment_name: String,
    pub environments: Vec<Environment>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Environment {
    pub name: String,
    pub default_cluster_name: String,
    pub clusters: Vec<Cluster>,
}

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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ServerGroup {
    pub name: String,
    pub servers: Vec<Server>,
}

// https://docs.rs/postgres/latest/postgres/config/struct.Config.html#
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct Server {
    pub host: String,
    pub port: Option<i32>,
    pub db_name: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub connect_timeout_sec: Option<i32>,
}

impl Server {
    fn from(
        from: &Server,
        port: &Option<i32>,
        db_name: &Option<String>,
        user: &Option<String>,
        password: &Option<String>,
        connect_timeout_sec: &Option<i32>,
    ) -> Self {
        Self {
            host: from.host.clone(),
            port: if from.port.is_none() { port.clone() } else { from.port.clone() },
            db_name: if from.db_name.is_none() { db_name.clone() } else { from.db_name.clone() },
            user: if from.user.is_none() { user.clone() } else { from.user.clone() },
            password: if from.password.is_none() { password.clone() } else { from.password.clone() },
            connect_timeout_sec: if from.connect_timeout_sec.is_none() { connect_timeout_sec.clone() } else { from.connect_timeout_sec.clone() },
        }
    }

    pub fn set_db_name(&mut self, db_name: String) {
        self.db_name = Some(db_name);
    }
    pub fn _get_db_name(&self) -> Option<String> {
        match &self.db_name {
            Some(db_name) => { Some(db_name.clone()) }
            None => { None }
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let connection_string = format!(
            "host={} port={} dbname={} user={} password={} connect_timeout={} application_name=taco",
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