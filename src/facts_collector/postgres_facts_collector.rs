use anyhow::Result;
use chrono::{DateTime, Duration, Local};
use std::net::IpAddr;
use tokio_postgres::NoTls;
use tokio_postgres::types::{Oid, PgLsn};

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

#[derive(Debug)]
pub struct PgStatWalReceiverResult {
    pid: Option<i32>,
    status: Option<String>,
    receive_start_lsn: Option<PgLsn>,
    receive_start_tli: Option<i32>,
    written_lsn: Option<PgLsn>,
    flushed_lsn: Option<PgLsn>,
    received_tli: Option<i32>,
    last_msg_send_time: Option<DateTime<Local>>,
    last_msg_receipt_time: Option<DateTime<Local>>,
    latest_end_lsn: Option<PgLsn>,
    latest_end_time: Option<DateTime<Local>>,
    slot_name: Option<String>,
    sender_host: Option<String>,
    sender_port: Option<i32>,
    conninfo: Option<String>,
}

pub struct PostgresFactsCollector<'a> {
    connection_string: &'a str,
}

impl<'a> PostgresFactsCollector<'a> {
    pub fn new(connection_string: &'a str) -> Self {
        PostgresFactsCollector { connection_string }
    }

    pub async fn check_pg_stat_replication(&self) -> Result<Vec<PgStatReplicationResult>> {
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
            result.push(PgStatReplicationResult {
                pid: row.get(0),
                usesysid: row.get(1),
                usename: row.get(2),
                application_name: row.get(3),
                client_addr: row.get(4),
                client_hostname: row.get(5),
                client_port: row.get(6),
                backend_start: row.get(7),
                backend_xmin: None,
                state: row.get(9),
                sent_lsn: row.get(10),
                write_lsn: row.get(11),
                flush_lsn: row.get(12),
                replay_lsn: row.get(13),
                write_lag: None,
                flush_lag: None,
                replay_lag: None,
                sync_priority: row.get(17),
                sync_state: row.get(18),
                reply_time: row.get(19),
            });
        }
        Ok(result)
    }

    pub async fn check_pg_stat_wal_receiver(&self) -> Result<Vec<PgStatWalReceiverResult>> {
        let (client, connection) = tokio_postgres::connect(&self.connection_string, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let rows = client
            .query("SELECT * FROM pg_stat_wal_receiver;", &[])
            .await?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let mut result: Vec<PgStatWalReceiverResult> = Vec::new();
        for row in rows {
            result.push(PgStatWalReceiverResult {
                pid: row.get(0),
                status: row.get(1),
                receive_start_lsn: row.get(2),
                receive_start_tli: row.get(3),
                written_lsn: row.get(4),
                flushed_lsn: row.get(5),
                received_tli: row.get(6),
                last_msg_send_time: row.get(7),
                last_msg_receipt_time: row.get(8),
                latest_end_lsn: row.get(9),
                latest_end_time: row.get(10),
                slot_name: row.get(11),
                sender_host: row.get(12),
                sender_port: row.get(13),
                conninfo: row.get(14),
            })
        }
        Ok(result)
    }
}
