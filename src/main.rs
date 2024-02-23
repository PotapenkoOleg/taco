mod version;

mod clap_parser;
mod inventory;
mod settings;
mod secrets;

use std::collections::HashMap;
use std::fmt::{Debug};
use tokio::task::JoinSet;
use tokio_postgres::{NoTls, Error};
use std::io::{self, BufRead};
use std::process;
use std::sync::{Arc, Mutex};
use clap::Parser;
use colored::Colorize;
use prettytable::{Cell, Row, Table};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use crate::clap_parser::Args;
use crate::inventory::inventory_manager::{InventoryManager, Server};
use crate::secrets::secrets_manager::SecretsManager;
use crate::settings::settings_manager::SettingsManager;

use crate::version::{COPYRIGHT, COPYRIGHT_YEARS, LICENSE, PRODUCT_NAME, VERSION_ALIAS, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH};

#[tokio::main]
async fn main() {
    print_separator();
    print_banner();
    print_separator();
    let args = Args::parse();
    let inventory_manager = load_inventory_file(&args.inventory);
    let settings_manager = load_settings_file(&args.settings);
    let secrets_manager = load_secrets_file(&args.secrets);
    print_separator();
    let mut history: Vec<String> = Vec::new();
    let settings = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let command = line.unwrap();
        let preprocessed_command = command.to_lowercase().trim().to_string();
        if preprocessed_command.cmp(&"exit".to_string()).is_eq() {
            process::exit(0);
        }
        if preprocessed_command.cmp(&"history".to_string()).is_eq() {
            println!("HISTORY");
            for (index, value) in history.iter().enumerate() {
                println!("{}: {}", index, value)
            }
            continue;
        }
        if preprocessed_command.cmp(&"help".to_string()).is_eq() {
            println!("FORMAT: <SERVER_GROUP><SEPARATOR><COMMAND>");
            println!("\"?\" - separator for query");
            println!("\"!\" - separator for command");
            println!("Examples: ");
            println!("primary ? select version();");
            println!("dr ! create extension citus;");
            continue;
        }
        if preprocessed_command.starts_with("batch") {
            // TODO:
            println!("{}", "BATCH STARTED".yellow());
            continue;
        }
        if preprocessed_command.starts_with("use") {
            let parts = preprocessed_command.split(" ");
            let parts_vec: Vec<&str> = parts.collect();
            println!("USING DB <{}>", parts_vec[1]);
            { // this block for mutex release
                let mut settings_lock = settings.lock().unwrap();
                settings_lock.insert("db".to_string(), parts_vec[1].to_string());
            }
            continue;
        }
        history.push(command.clone());
        process_request(command, &inventory_manager, &settings).await;
    }
}

fn print_banner() {
    println!("{}",
             format!("{} version {}.{}.{} ({})",
                     PRODUCT_NAME,
                     VERSION_MAJOR,
                     VERSION_MINOR,
                     VERSION_PATCH,
                     VERSION_ALIAS
             )
    );
    println!("License: {}", LICENSE);
    println!("Copyright © {}. {}.", COPYRIGHT, COPYRIGHT_YEARS);
}

fn print_separator() {
    let template = "*";
    let n = 80;
    let repeated_string = template.repeat(n);
    println!("{}", repeated_string);
}

fn load_inventory_file(inventory_file_name: &str) -> InventoryManager {
    print!("Loading inventory file: <{}> ... ", inventory_file_name);
    let mut inventory_manager = InventoryManager::new(&inventory_file_name);
    inventory_manager.load_inventory_from_file().expect("TODO: panic message");
    print!("DONE\n");
    inventory_manager
}

fn load_settings_file(settings_file_name: &str) -> SettingsManager {
    print!("Loading settings file: <{}> ... ", settings_file_name);
    let settings_manager = SettingsManager::new(&settings_file_name);
    print!("DONE\n");
    settings_manager
}

fn load_secrets_file(secrets_file_name: &str) -> SecretsManager {
    print!("Loading secrets file: <{secrets_file_name}> ... ");
    let secrets_manager = SecretsManager::new(&secrets_file_name);
    print!("DONE\n");
    secrets_manager
}

async fn process_request(
    command: String,
    inventory_manager: &InventoryManager,
    settings: &Arc<Mutex<HashMap<String, String>>>,
) {
    let request_type = get_request_type(&command);
    let get_raw_command_result = get_raw_command(&command, &request_type);
    let raw_server_group = get_raw_command_result.0;
    let raw_command = get_raw_command_result.1;

    print_separator();
    println!("Processing: [{}]", &raw_command.red());
    print_separator();

    let servers = inventory_manager.get_servers(&raw_server_group);

    let (tx, mut rx) = mpsc::channel(32);

    tokio::spawn(async move {
        while let result = rx.recv().await {
            match result {
                Some(printable_result) => {
                    print!("{}", printable_result);
                }
                None => {}
            }
        }
    });

    let mut set = JoinSet::new();

    for server in servers {
        let command_clone = raw_command.clone();
        let request_type_clone = request_type.clone();
        let settings_clone = settings.clone();
        let tx_clone = tx.clone();
        set.spawn(async move {
            match request_type_clone {
                RequestType::Query => { process_query(server, command_clone, settings_clone, tx_clone).await }
                RequestType::Command => { process_command(server, command_clone, settings_clone, tx_clone).await }
                _ => { Ok(0u64) }
            }
        });
    }

    let mut total: u64 = 0;
    while let Some(res) = set.join_next().await {
        total += res.unwrap().unwrap();
    }
    print_separator();
    println!("Total rows: {}", total);
    print_separator();
}

fn get_request_type(command: &String) -> RequestType {
    if command.contains("?") {
        return RequestType::Query;
    }
    if command.contains("!") {
        return RequestType::Command;
    }
    RequestType::Unknown
}

fn get_raw_command(command: &String, request_type: &RequestType) -> (String, String) {
    let request_separator = match request_type {
        RequestType::Query => "?".to_string(),
        RequestType::Command => "!".to_string(),
        _ => { "".to_string() }
    };

    let parts = command.split(&request_separator);
    let mut raw_parts: Vec<String> = parts.into_iter()
        .map(|x| x.to_string().trim().to_lowercase())
        .collect();

    return (raw_parts.remove(0), raw_parts.remove(0));
}

async fn process_query(
    mut server: Server,
    query: String,
    settings: Arc<Mutex<HashMap<String, String>>>,
    tx: Sender<String>,
) -> Result<u64, Error> {
    { // this block for mutex release
        let settings_lock = settings.lock().unwrap();
        match settings_lock.get(&"db".to_string()) {
            Some(db_name) => {
                server.set_db_name(db_name.clone());
            }
            _ => {}
        }
    }

    let (client, connection) =
        tokio_postgres::connect(&server.to_string(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query(&query, &[])
        .await?;

    let mut table = Table::new();

    // https://stackoverflow.com/questions/62755435/how-to-enumerate-over-columns-with-tokio-postgres-when-the-field-types-are-unkno
    for (row_index, row) in rows.iter().enumerate() {
        let mut row_vec: Vec<Cell> = Vec::new();
        row_vec.push(Cell::new(&*format!("{}", row_index)));
        for (col_index, column) in row.columns().iter().enumerate() {
            let col_type: String = column.type_().to_string();
            if col_type == "varchar" || col_type == "text" {
                let value: &str = row.get(col_index);
                row_vec.push(Cell::new(value));
            }
        }
        table.add_row(Row::new(row_vec));
    }

    let mut result = String::new();
    result.push_str(&format!("\n[{}:{}] \n", &server.host, &server.db_name.unwrap()));
    result.push_str(&format!("{}\n", table.to_string()));
    tx.send(result).await.expect("TODO: panic message");

    Ok(rows.len() as u64)
}

async fn process_command(
    mut server: Server,
    command: String,
    settings: Arc<Mutex<HashMap<String, String>>>,
    tx: Sender<String>,
) -> Result<u64, Error> {
    { // this block for mutex release
        let settings_lock = settings.lock().unwrap();
        match settings_lock.get(&"db".to_string()) {
            Some(db_name) => {
                server.set_db_name(db_name.clone());
            }
            _ => {}
        }
    }

    let (client, connection) =
        tokio_postgres::connect(&server.to_string(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let query = &command;
    let statement = client.prepare(query).await?;
    let rows = client.execute(&statement, &[]).await?;

    let mut result = String::new();
    result.push_str(&format!("[{}:{}]: rows {}", &server.host, &server.db_name.unwrap(), rows));
    tx.send(result).await.expect("TODO: panic message");

    Ok(rows)
}

#[derive(Clone, Debug)]
enum RequestType {
    Query,
    Command,
    Unknown,
}