mod version;

mod clap_parser;
mod inventory;
mod settings;
mod secrets;

use std::any::Any;
use tokio::task::JoinSet;
use tokio_postgres::{NoTls, Error};
use tokio_postgres::types::ToSql;
use std::io::{self, BufRead, Read, Write};
use std::process;
use clap::Parser;
use colored::Colorize;
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
        history.push(command.clone());
        process_command(command, &inventory_manager).await;
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

async fn process_command(command: String, inventory_manager: &InventoryManager) {
    print_separator();
    println!("Processing: [{}]", &command.red());
    print_separator();

    let mut connection_strings = inventory_manager.get_connection_strings();

    //println!("{:?}", connection_strings);

    let mut set = JoinSet::new();

    for i in 0..connection_strings.len() {
        let x = connection_strings.pop();
        let y = command.clone();
        set.spawn(async move {
            pg_main(i, x.unwrap(), y).await
        });
    }

    while let Some(res) = set.join_next().await {
        let idx = res.unwrap();
        //println!("{:?}", idx);
    }
    print_separator();
}

async fn pg_main(i: usize, connection_string: String, command: String) -> Result<u64, Error> {
    //print_separator();
    let (client, connection) =
        tokio_postgres::connect(&connection_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    // let task_id = i.clone() as i32;
    // let id = i.clone() as i32;
    // let value = i.clone() as i32;
    //
    // let query = "INSERT INTO test (id, value) VALUES ($1, $2);";
    // let statement = client.prepare(query).await?;
    // let res = client.execute(&statement, &[&id, &value]).await?;
    // println!("Task # {} - {}", task_id, res);

    let rows = client
        .query(&command, &[])
        .await?;

    // let d = &rows[0];
    // let p = d.columns().get(0).unwrap()
    let value: &str = rows[0].get(0);
    println!("{}: [{}] {}", &i, &connection_string.green(), value.yellow());


    Ok(rows.len() as u64)
}