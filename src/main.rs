mod version;

use std::any::Any;
use tokio::task::JoinSet;
use tokio_postgres::{NoTls, Error};
use tokio_postgres::types::ToSql;
use std::io::{self, BufRead};
use std::process;
use colored::Colorize;
use crate::version::{COPYRIGHT, COPYRIGHT_YEARS, LICENSE, PRODUCT_NAME, VERSION_ALIAS, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH};

#[tokio::main]
async fn main() {
    print_separator();
    print_header();
    print_separator();
    load_inventory_file();
    print_separator();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let command = line.unwrap();
        if command.cmp(&"exit".to_string()).is_eq() {
            process::exit(0);
        }
        process_command(command).await;
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

fn load_inventory_file() {
    println!("{}", "Loading inventory file: inventory.taco ... DONE".green());
    println!("{}", "Loading settings file: settings.taco ... DONE".green());
    println!("{}", "Loading secrets file: secrets.taco ... DONE".green());
}

async fn process_command(command: String) {
    println!("PROCESSING: {}", &command);

    let mut connection_strings = vec![
        "host=localhost port=5432 user=postgres password=postgres".to_string(),
        "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
        // "host=localhost port=5432 user=postgres password=postgres".to_string(),
    ];
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
    println!("{}: [{}] {:?}", &i, &connection_string, value);
    //print_separator();

    Ok(rows.len() as u64)
}