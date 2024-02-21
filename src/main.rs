mod version;

mod clap_parser;
mod inventory;
mod settings;
mod secrets;

use std::any::Any;
use std::fmt::{Debug, format};
use tokio::task::JoinSet;
use tokio_postgres::{NoTls, Error};
use tokio_postgres::types::ToSql;
use std::io::{self, BufRead, Read, Write};
use std::process;
use clap::Parser;
use colored::Colorize;
use prettytable::{Cell, Row, row, Table};
use serde::{Deserialize, Serialize};
use crate::clap_parser::Args;
use crate::inventory::inventory_manager::InventoryManager;
use crate::secrets::secrets_manager::SecretsManager;
use crate::settings::settings_manager::SettingsManager;

use crate::version::{COPYRIGHT, COPYRIGHT_YEARS, LICENSE, PRODUCT_NAME, VERSION_ALIAS, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH};

#[tokio::main]
async fn main() {
    print_separator();
    print_header();
    print_separator();
    let args = Args::parse();
    let inventory_manager = load_inventory_file(&args.inventory);
    let settings_manager = load_settings_file(&args.settings);
    let secrets_manager = load_secrets_file(&args.secrets);
    let mut history: Vec<String> = Vec::new();
    print_separator();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let command = line.unwrap();
        if command.to_lowercase().trim().cmp(&"exit".to_string()).is_eq() {
            process::exit(0);
        }
        if command.to_lowercase().trim().cmp(&"history".to_string()).is_eq() {
            for (index, value) in history.iter().enumerate() {
                println!("{}: {}", index, value)
            }
            continue;
        }
        if command.to_lowercase().trim().cmp(&"help".to_string()).is_eq() {
            println!("FORMAT: <SERVER_GROUP><SEPARATOR><COMMAND>");
            println!("\"?\" - separator for query");
            println!("\"!\" - separator for command");
            println!("Examples: ");
            println!("primary ? select version();");
            println!("dr ! create extension citus;");
            continue;
        }
        history.push(command.clone());
        process_request(command, &inventory_manager).await;
    }
}

fn print_header() {
    let version =
        format!("{} version {}.{}.{} ({})",
                PRODUCT_NAME,
                VERSION_MAJOR,
                VERSION_MINOR,
                VERSION_PATCH,
                VERSION_ALIAS
        ).red().bold();
    println!("{}", version);

    let license = format!("License: {}", LICENSE).red();
    println!("{}", license);

    let copyright = format!("Copyright © {}. {}.", COPYRIGHT, COPYRIGHT_YEARS).red();
    println!("{}", copyright);
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

async fn process_request(command: String, inventory_manager: &InventoryManager) {
    let request_type = get_request_type(&command);
    let get_raw_command_result = get_raw_command(&command, &request_type);
    let raw_server_group = get_raw_command_result.0;
    let raw_command = get_raw_command_result.1;

    print_separator();
    println!("Processing: [{}]", &raw_command.red());
    print_separator();

    let mut connection_strings = inventory_manager.get_connection_strings(&raw_server_group);

    let mut set = JoinSet::new();

    for i in 0..connection_strings.len() {
        let connection_string = connection_strings.pop();
        let command_clone = raw_command.clone();
        let request_type_clone = request_type.clone();
        set.spawn(async move {
            match request_type_clone {
                RequestType::Query => { process_query(i, connection_string.unwrap(), command_clone).await }
                RequestType::Command => { process_command(i, connection_string.unwrap(), command_clone).await }
                _ => { Ok(0u64) }
            }
        });
    }

    let mut total: u64 = 0;
    while let Some(res) = set.join_next().await {
        //total += res.unwrap().unwrap();
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

async fn process_query(i: usize, connection_string: String, query: String) -> Result<u64, Error> {
    let (client, connection) =
        tokio_postgres::connect(&connection_string, NoTls).await?;

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

    println!("{}: [{}]", &i, &connection_string.green(), );
    print!("{}", table.to_string());

    Ok(rows.len() as u64)
}

async fn process_command(i: usize, connection_string: String, command: String) -> Result<u64, Error> {
    let (client, connection) =
        tokio_postgres::connect(&connection_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let task_id = i as i32;

    let query = &command;
    let statement = client.prepare(query).await?;
    let rows = client.execute(&statement, &[]).await?;
    println!("Task # {} - {}", task_id, rows);

    Ok(rows)
}

#[derive(Clone, Debug)]
enum RequestType {
    Query,
    Command,
    Unknown,
}