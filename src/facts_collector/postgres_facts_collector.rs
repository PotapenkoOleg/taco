use crate::shared::pg_stat_replication_result::PgStatReplicationResult;
use crate::shared::pg_stat_wal_receiver_result::PgStatWalReceiverResult;
use anyhow::Result;
use tokio_postgres::NoTls;

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
