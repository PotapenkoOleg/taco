use chrono::{DateTime, Duration, Local};
use std::net::IpAddr;
use tokio_postgres::types::{Oid, PgLsn};
use tokio_postgres::{Error, NoTls};

#[derive(Debug)]
pub struct PgStatReplicationResult {
    pid: Option<i32>,
    usesysid: Option<Oid>,
    usename: Option<String>,
    application_name: Option<String>,
    client_addr: Option<IpAddr>,
    client_hostname: Option<String>,
    client_port: Option<i32>,
    backend_start: Option<DateTime<Local>>,
    backend_xmin: Option<i32>,
    state: Option<String>,
    sent_lsn: Option<PgLsn>,
    write_lsn: Option<PgLsn>,
    flush_lsn: Option<PgLsn>,
    replay_lsn: Option<PgLsn>,
    write_lag: Option<Duration>,
    flush_lag: Option<Duration>,
    replay_lag: Option<Duration>,
    sync_priority: Option<i32>,
    sync_state: Option<String>,
    reply_time: Option<DateTime<Local>>,
}

pub struct PostgresFactsCollector<'a> {
    connection_string: &'a str,
}

impl<'a> PostgresFactsCollector<'a> {
    pub fn new(connection_string: &'a str) -> Self {
        PostgresFactsCollector { connection_string }
    }

    pub async fn check_pg_stat_replication(&self) -> Result<Vec<PgStatReplicationResult>, Error> {
        let (client, connection) = tokio_postgres::connect(&self.connection_string, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let rows = client
            .query("SELECT * FROM pg_stat_replication;", &[])
            .await?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let mut result: Vec<PgStatReplicationResult> = Vec::new();
        for row in rows {
            let pid: Option<i32> = row.get(0);
            let usesysid: Option<Oid> = row.get(1);
            let usename: Option<&str> = row.get(2);
            let application_name: Option<&str> = row.get(3);
            let client_addr: Option<IpAddr> = row.get(4);
            let client_hostname: Option<&str> = row.get(5);
            let client_port: Option<i32> = row.get(6);
            let backend_start: Option<DateTime<Local>> = row.get(7);
            //let backend_xmin: Option<i32> = row.get(8); //*
            let state: Option<&str> = row.get(9);
            let sent_lsn: Option<PgLsn> = row.get(10);
            let write_lsn: Option<PgLsn> = row.get(11);
            let flush_lsn: Option<PgLsn> = row.get(12);
            let replay_lsn: Option<PgLsn> = row.get(13);
            //let write_lag: Option<Duration> =  row.get(14); // *
            //let flush_lag: Option<Duration> = row.get(15); // *
            //let replay_lag: Option<Duration> = row.get(16); // *
            let sync_priority: Option<i32> = row.get(17);
            let sync_state: Option<&str> = row.get(18);
            let reply_time: Option<DateTime<Local>> = row.get(19); // *

            result.push(PgStatReplicationResult {
                pid,
                usesysid,
                usename: Some(usename.unwrap_or("").to_string()),
                application_name: Some(application_name.unwrap_or("").to_string()),
                client_addr,
                client_hostname: Some(client_hostname.unwrap_or("").to_string()),
                client_port,
                backend_start,
                backend_xmin: None,
                state: Some(state.unwrap_or("").to_string()),
                sent_lsn,
                write_lsn,
                flush_lsn,
                replay_lsn,
                write_lag: None,
                flush_lag: None,
                replay_lag: None,
                sync_priority,
                sync_state: Some(sync_state.unwrap_or("").to_string()),
                reply_time,
            });
        }
        Ok(result)
    }
}
