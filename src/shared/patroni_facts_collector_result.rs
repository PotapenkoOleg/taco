pub struct PatroniFactsCollectorResult {
    pub healthy: Option<bool>,
    pub is_primary: Option<bool>,
    pub is_replica: Option<bool>,
    pub replica_has_no_lag: Option<bool>,
    pub is_read_write: Option<bool>,
    pub is_read_only: Option<bool>,
    pub is_standby_leader: Option<bool>,
    pub is_sync_standby: Option<bool>,
    pub is_async_standby: Option<bool>,
}