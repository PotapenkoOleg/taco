use std::net::IpAddr;
use chrono::{DateTime, Duration, Local};
use tokio_postgres::types::{Oid, PgLsn};

#[derive(Debug)]
pub struct PgStatReplicationResult {
    pub pid: Option<i32>,
    pub usesysid: Option<Oid>,
    pub usename: Option<String>,
    pub application_name: Option<String>,
    pub client_addr: Option<IpAddr>,
    pub client_hostname: Option<String>,
    pub client_port: Option<i32>,
    pub backend_start: Option<DateTime<Local>>,
    pub backend_xmin: Option<i32>,
    pub state: Option<String>,
    pub sent_lsn: Option<PgLsn>,
    pub write_lsn: Option<PgLsn>,
    pub flush_lsn: Option<PgLsn>,
    pub replay_lsn: Option<PgLsn>,
    pub write_lag: Option<Duration>,
    pub flush_lag: Option<Duration>,
    pub replay_lag: Option<Duration>,
    pub sync_priority: Option<i32>,
    pub sync_state: Option<String>,
    pub reply_time: Option<DateTime<Local>>,
}