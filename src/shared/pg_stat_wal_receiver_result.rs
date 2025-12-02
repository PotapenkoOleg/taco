use chrono::{DateTime, Local};
use tokio_postgres::types::PgLsn;

#[derive(Debug)]
pub struct PgStatWalReceiverResult {
    pub pid: Option<i32>,
    pub status: Option<String>,
    pub receive_start_lsn: Option<PgLsn>,
    pub receive_start_tli: Option<i32>,
    pub written_lsn: Option<PgLsn>,
    pub flushed_lsn: Option<PgLsn>,
    pub received_tli: Option<i32>,
    pub last_msg_send_time: Option<DateTime<Local>>,
    pub last_msg_receipt_time: Option<DateTime<Local>>,
    pub latest_end_lsn: Option<PgLsn>,
    pub latest_end_time: Option<DateTime<Local>>,
    pub slot_name: Option<String>,
    pub sender_host: Option<String>,
    pub sender_port: Option<i32>,
    pub conninfo: Option<String>,
}