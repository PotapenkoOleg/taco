mod version;

mod clap_parser;
mod facts_collector;
mod inventory;

use crate::clap_parser::Args;
use crate::inventory::inventory_manager::{InventoryManager, Server};
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};
use clap::Parser;
use colored::Colorize;
use prettytable::{Cell, Row, Table};
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;
use tokio_postgres::types::{FromSql, Oid, Type};
use tokio_postgres::{Error, NoTls};
use uuid::Uuid;

use crate::version::{
    COPYRIGHT, COPYRIGHT_YEARS, LICENSE, LINK, PRODUCT_NAME, VERSION_ALIAS, VERSION_MAJOR,
    VERSION_MINOR, VERSION_PATCH,
};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    print_separator();
    print_banner();
    print_separator();
    let inventory_manager = load_inventory_file(&args.inventory).await;
    print_separator();
    let mut history: Vec<String> = Vec::new();
    let settings = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    {
        // this block for mutex release
        let mut settings_lock = settings.lock().unwrap();
        settings_lock.insert("db".to_string(), "postgres".to_string());
    }
    loop {
        let mut current_db: Option<String> = None;
        {
            // this block for mutex release
            let settings_lock = settings.lock().unwrap();
            match settings_lock.get(&"db".to_string()) {
                Some(db_name) => {
                    current_db = Some(db_name.clone());
                }
                _ => {}
            }
        }
        let _ =
            io::stdout().write(format!("[{}] > ", current_db.unwrap_or("".to_string())).as_ref());
        let _ = io::stdout().flush();
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let preprocessed_command = command.to_lowercase().trim().to_string();
        if preprocessed_command.cmp(&"exit".to_string()).is_eq() {
            println!("{}", "BYE-BYE!".yellow());
            process::exit(0);
        }
        if preprocessed_command.cmp(&"history".to_string()).is_eq() {
            println!("{}", "HISTORY".yellow());
            for (index, value) in history.iter_mut().enumerate() {
                trim_newline(value);
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
            let parts = preprocessed_command.split(" ");
            let parts_vec: Vec<&str> = parts.collect();
            if parts_vec[1].eq("start") {
                println!("{}", "BATCH STARTED".yellow());
            }
            if parts_vec[1].eq("end") {
                println!("{}", "BATCH ENDED".yellow());
            }
            // TODO:
            continue;
        }
        if preprocessed_command.starts_with("use") {
            let parts = preprocessed_command.split(" ");
            let parts_vec: Vec<&str> = parts.collect();
            println!("{}", format!("USING DB <{}>", parts_vec[1]).yellow());
            {
                // this block for mutex release
                let mut settings_lock = settings.lock().unwrap();
                settings_lock.insert("db".to_string(), parts_vec[1].to_string());
            }
            continue;
        }
        if preprocessed_command.starts_with("show") {
            let parts = preprocessed_command.split(" ");
            let parts_vec: Vec<&str> = parts.collect();
            println!("{}", format!("SHOW DATA TYPES <{}>", parts_vec[2]).yellow());
            {
                // this block for mutex release
                let mut settings_lock = settings.lock().unwrap();
                settings_lock.insert("show_data_types".to_string(), parts_vec[2].to_string());
            }
            continue;
        }
        if preprocessed_command.is_empty() {
            println!("{}", "UNKNOWN REQUEST TYPE".red());
            continue;
        }
        let request_type = get_request_type(&command);
        match &request_type {
            RequestType::Unknown => {
                println!("{}", "UNKNOWN REQUEST TYPE".red());
                continue;
            }
            _ => {
                history.push(command.clone());
                let get_raw_command_result = get_raw_command(&command, &request_type);
                let raw_server_group = get_raw_command_result.0;
                let raw_command = get_raw_command_result.1;
                let servers = inventory_manager.get_servers(&raw_server_group);
                let settings_clone = settings.clone();
                let handle = tokio::spawn(async move {
                    process_request(raw_command, request_type, servers, settings_clone).await
                });
                handle.await.unwrap();
            }
        }
    }
}

fn print_banner() {
    println!(
        "{}",
        format!(
            "{} version {}.{}.{} ({})",
            PRODUCT_NAME, VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH, VERSION_ALIAS
        )
    );
    println!("License: {}", LICENSE);
    println!("Link: {}", LINK);
    println!("Copyright © {}. {}", COPYRIGHT, COPYRIGHT_YEARS);
}

fn build_separator() -> String {
    let template = "*";
    let n = 80;
    template.repeat(n)
}

fn print_separator() {
    println!("{}", build_separator());
}

async fn load_inventory_file(inventory_file_name: &str) -> InventoryManager {
    println!("Loading Inventory File: <{}> ", inventory_file_name);
    let mut inventory_manager = InventoryManager::new(&inventory_file_name);
    let result = inventory_manager.load_inventory_from_file().await;
    if result.is_err() {
        eprintln!("{}", result.err().unwrap().to_string().red());
        process::exit(1);
    }
    println!("DONE Loading Inventory File");
    inventory_manager
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

async fn process_request(
    //command: String,
    raw_command: String,
    request_type: RequestType,
    servers: HashSet<Server>,
    // inventory_manager: &InventoryManager,
    settings: Arc<Mutex<HashMap<String, String>>>,
) {
    print_separator();
    println!("Processing: [{}]", &raw_command.green());
    print_separator();

    let (tx, mut rx) = mpsc::channel(32);

    let mut set = JoinSet::new();

    for server in servers {
        let command_clone = raw_command.clone();
        let request_type_clone = request_type.clone();
        let settings_clone = settings.clone();
        let tx_clone = tx.clone();
        set.spawn(async move {
            match request_type_clone {
                RequestType::Query => {
                    process_query(server, command_clone, settings_clone, tx_clone).await
                }
                RequestType::Command => {
                    process_command(server, command_clone, settings_clone, tx_clone).await
                }
                _ => Ok(0u64),
            }
        });
    }

    tokio::spawn(async move {
        while let result = rx.recv().await {
            match result {
                Some(printable_result) => {
                    print!("{}", printable_result);
                }
                None => {}
            }
        }
        Ok::<u64, Error>(0u64)
    });

    let mut total: u64 = 0;
    while let Some(res) = set.join_next().await {
        total += res.unwrap().unwrap(); // TODO
    }

    let mut result = String::new();
    result.push_str(&format!("\n{}\n", build_separator()));
    result.push_str(&format!("Total rows: {}", total));
    result.push_str(&format!("\n{}\n", build_separator()));
    if tx.send(result).await.as_ref().is_err() {
        eprintln!("{}", "ERROR SENDING RESULT TO PRINTER THREAD".red());
    }
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
        _ => "".to_string(),
    };

    let parts = command.split(&request_separator);
    let mut raw_parts: Vec<String> = parts
        .into_iter()
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
    let mut show_data_types = false;
    {
        // this block for mutex release
        let settings_lock = settings.lock().unwrap();
        match settings_lock.get(&"db".to_string()) {
            Some(db_name) => {
                server.set_db_name(db_name.clone());
            }
            _ => {}
        }
        match settings_lock.get(&"show_data_types".to_string()) {
            Some(show_dt) => {
                if show_dt.eq("true") {
                    show_data_types = true;
                }
                if show_dt.eq("false") {
                    show_data_types = false;
                }
            }
            _ => {
                show_data_types = true;
            }
        }
    }

    let connect_result = tokio_postgres::connect(&server.to_string(), NoTls).await;
    if connect_result.as_ref().is_err() {
        let mut result = String::new();
        result.push_str(&format!(
            "\n[{}:{}] \n",
            &server.host,
            &server.db_name.unwrap()
        ));
        result.push_str(&*connect_result.as_ref().err().unwrap().to_string());
        result.push_str(&*"\n".to_string());
        if tx.send(result.clone()).await.is_err() {
            eprintln!("{}", result.red());
        }
        return Ok(0u64);
    }

    let (client, connection) = connect_result.unwrap();
    tokio::spawn(async move {
        if connection.await.as_ref().is_err() {
            eprintln!("{}", "ERROR OPEN CONNECTION".red());
        }
    });

    let rows_result = client.query(&query, &[]).await;
    if rows_result.as_ref().is_err() {
        let mut result = String::new();
        result.push_str(&format!(
            "\n[{}:{}] \n",
            &server.host,
            &server.db_name.unwrap()
        ));
        result.push_str(&*rows_result.as_ref().err().unwrap().to_string());
        result.push_str(&*"\n".to_string());
        if tx.send(result.clone()).await.is_err() {
            eprintln!("{}", result.red());
        }
        return Ok(0u64);
    }

    let rows = rows_result.unwrap();

    if rows.len() == 0 {
        return Ok(0u64);
    }

    let mut table = Table::new();

    let mut row_vec: Vec<Cell> = Vec::new();
    row_vec.push(Cell::new(&""));
    for column in rows[0].columns().iter() {
        let mut column_header = String::new();
        column_header.push_str(column.name());
        if show_data_types {
            column_header.push(':');
            column_header.push_str(&*column.type_().to_string());
        }
        row_vec.push(Cell::new(&*column_header));
    }
    table.add_row(Row::new(row_vec));
    for (row_index, row) in rows.iter().enumerate() {
        let mut row_vec: Vec<Cell> = Vec::new();
        row_vec.push(Cell::new(&*format!("{}", row_index)));
        for (col_index, column) in row.columns().iter().enumerate() {
            // https://www.postgresql.org/docs/current/datatype.html
            let col_type: String = column.type_().to_string();

            // region Numeric Types
            // https://www.postgresql.org/docs/current/datatype-numeric.html
            if col_type == "int2" || col_type == "smallint" || col_type == "smallserial" {
                let value: i16 = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "int4" || col_type == "int" || col_type == "serial" {
                let value: i32 = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "int8" || col_type == "bigint" || col_type == "bigserial" {
                let value: i64 = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "decimal" || col_type == "numeric" {
                let value: Decimal = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "real" || col_type == "float4" {
                let value: f32 = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "double precision" || col_type == "float8" {
                let value: f64 = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            // endregion

            // region Monetary Types
            // https://www.postgresql.org/docs/current/datatype-money.html
            if col_type == "money" {
                // TODO:
                //let value: Money = row.get(col_index);
                //row_vec.push(Cell::new(&*value.to_string()));
                row_vec.push(Cell::new("?money?"));
                continue;
            }
            // endregion

            // region Character Types
            // https://www.postgresql.org/docs/current/datatype-character.html
            if col_type == "varchar"
                || col_type == "text"
                || col_type == "bpchar"
                || col_type == "character"
                || col_type == "char"
            {
                // TODO: char type
                // SELECT attalign FROM pg_attribute WHERE attrelid = 'test'::regclass;
                let value: &str = row.get(col_index);
                row_vec.push(Cell::new(value));
                continue;
            }
            // endregion

            // region Binary Data Types
            // https://www.postgresql.org/docs/current/datatype-binary.html
            if col_type == "bytea" {
                // TODO:
                row_vec.push(Cell::new("?bytea?"));
                continue;
            }
            // endregion

            // region Date/Time Types
            // https://www.postgresql.org/docs/current/datatype-datetime.html
            if col_type == "timestamp" {
                let value: NaiveDateTime = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "timestamptz" {
                let value: DateTime<Local> = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "time" {
                let value: NaiveTime = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "timetz" {
                let value: DateTime<Local> = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                // TODO:
                //row_vec.push(Cell::new("?timetz?"));
                continue;
            }
            if col_type == "date" {
                let value: NaiveDate = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            if col_type == "interval" {
                // let value: IntervalWrapper = row.get(col_index);
                // row_vec.push(Cell::new(&*value.to_string()));
                // TODO:
                row_vec.push(Cell::new("?interval?"));
                continue;
            }
            // endregion

            // region Boolean Type
            // https://www.postgresql.org/docs/current/datatype-boolean.html
            if col_type == "boolean" || col_type == "bool" {
                let value: bool = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            // endregion

            // region UUID Type
            //https://www.postgresql.org/docs/current/datatype-uuid.html
            if col_type == "uuid" {
                let value: Uuid = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            // endregion

            // region Object Identifier Types
            // https://www.postgresql.org/docs/current/datatype-oid.html
            if col_type == "oid" {
                // SELECT attrelid,attname,atttypid,attlen,attnum,attcacheoff,atttypmod,attndims,attbyval,attnotnull,atthasdef,atthasmissing,attisdropped,attislocal,attinhcount,attstattarget,attcollation,attacl,attoptions,attfdwoptions,attmissingval FROM pg_attribute WHERE attrelid = 'test'::regclass;
                let value: Oid = row.get(col_index);
                row_vec.push(Cell::new(&*value.to_string()));
                continue;
            }
            // endregion

            // TODO: more types
            row_vec.push(Cell::new("?")); //placeholder for unknown types
        }
        table.add_row(Row::new(row_vec));
    }

    let mut result = String::new();
    result.push_str(&format!(
        "\n[{}:{}] \n",
        &server.host,
        &server.db_name.unwrap()
    ));
    result.push_str(&format!("{}\n", table.to_string()));
    if tx.send(result).await.as_ref().is_err() {
        eprintln!("{}", "ERROR SENDING RESULT TO PRINTER THREAD".red());
    }

    Ok(rows.len() as u64)
}

async fn process_command(
    mut server: Server,
    command: String,
    settings: Arc<Mutex<HashMap<String, String>>>,
    tx: Sender<String>,
) -> Result<u64, Error> {
    {
        // this block for mutex release
        let settings_lock = settings.lock().unwrap();
        match settings_lock.get(&"db".to_string()) {
            Some(db_name) => {
                server.set_db_name(db_name.clone());
            }
            _ => {}
        }
    }

    let connect_result = tokio_postgres::connect(&server.to_string(), NoTls).await;
    if connect_result.as_ref().is_err() {
        let mut result = String::new();
        result.push_str(&format!(
            "\n[{}:{}] \n",
            &server.host,
            &server.db_name.unwrap()
        ));
        result.push_str(&*connect_result.as_ref().err().unwrap().to_string());
        result.push_str(&*"\n".to_string());
        if tx.send(result.clone()).await.is_err() {
            eprintln!("{}", result.red());
        }
        return Ok(0u64);
    }

    let (client, connection) = connect_result.unwrap();
    tokio::spawn(async move {
        if connection.await.as_ref().is_err() {
            eprintln!("{}", "ERROR OPEN CONNECTION".red());
        }
    });

    let query = &command;
    let statement_result = client.prepare(query).await;
    if statement_result.as_ref().is_err() {
        let mut result = String::new();
        result.push_str(&format!(
            "\n[{}:{}] \n",
            &server.host,
            &server.db_name.unwrap()
        ));
        result.push_str(&*statement_result.as_ref().err().unwrap().to_string());
        result.push_str(&*"\n".to_string());
        if tx.send(result.clone()).await.is_err() {
            eprintln!("{}", result.red());
        }
        return Ok(0u64);
    }
    let statement = statement_result.unwrap();
    let rows_result = client.execute(&statement, &[]).await;
    if rows_result.as_ref().is_err() {
        let mut result = String::new();
        result.push_str(&format!(
            "\n[{}:{}] \n",
            &server.host,
            &server.db_name.unwrap()
        ));
        result.push_str(&*rows_result.as_ref().err().unwrap().to_string());
        result.push_str(&*"\n".to_string());
        if tx.send(result.clone()).await.is_err() {
            eprintln!("{}", result.red());
        }
        return Ok(0u64);
    }

    let rows = rows_result.unwrap();

    let mut result = String::new();
    result.push_str(&format!(
        "\n[{}:{}]: rows {}\n",
        &server.host,
        &server.db_name.unwrap(),
        rows
    ));
    if tx.send(result).await.as_ref().is_err() {
        eprintln!("{}", "ERROR SENDING RESULT TO PRINTER THREAD".red());
    }

    Ok(rows)
}

#[derive(Clone, Debug)]
enum RequestType {
    Query,
    Command,
    Unknown,
}

struct IntervalWrapper {}

impl<'a> FromSql<'a> for IntervalWrapper {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        match *ty {
            Type::INTERVAL => {
                let _str_value = std::str::from_utf8(raw)?;
                // Months: A 32-bit integer representing the number of months in the interval. This part accounts for the year and month components of the interval, where each year is considered to be 12 months.
                // Days: A 32-bit integer representing the number of days in the interval. This part is separate from the months and directly represents the days component of the interval.
                // Microseconds: A 64-bit integer representing the time of day component in microseconds. This allows for a precise representation of hours, minutes, seconds, and even fractions of a second within the interval.
                Ok(IntervalWrapper {})
            }
            _ => Err("Unsupported type")?,
        }
    }

    fn accepts(ty: &Type) -> bool {
        *ty == Type::INTERVAL
    }
}

#[tokio::test]
async fn test_query_data_types() {}
