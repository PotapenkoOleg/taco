use serde::{Deserialize, Serialize};
use std::fmt;

// https://docs.rs/postgres/latest/postgres/config/struct.Config.html#
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct Server {
    pub host: String,
    pub port: Option<i32>,
    pub db_name: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub connect_timeout_sec: Option<i32>,

    //region Runtime Information
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub is_node_online: Option<bool>,

    // region Facts Collector

    // region Postgres
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub postgres_is_leader: Option<bool>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub postgres_is_replica: Option<bool>,
    // endregion

    // region Citus
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub citus_is_leader_coordinator_node: Option<bool>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub citus_is_replica_coordinator_node: Option<bool>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub citus_is_leader_worker_node: Option<bool>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub citus_is_replica_worker_node: Option<bool>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub citus_is_active_worker_node: Option<bool>, // worker node could be inactive
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub citus_group_id:Option<i32>,
    // endregion

    // region Patroni
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub patroni_is_primary: Option<bool>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub patroni_is_replica: Option<bool>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub patroni_is_read_write: Option<bool>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub patroni_is_read_only: Option<bool>,
    // endregion
    // endregion
    // endregion
}

impl Server {
    pub fn from(
        from: &Server,
        defaults: (
            &Option<i32>,
            &Option<String>,
            &Option<String>,
            &Option<String>,
            &Option<i32>,
        ),
    ) -> Self {
        let (port, db_name, user, password, connect_timeout_sec) = defaults;
        Self {
            host: from.host.clone(),
            port: from.port.or_else(|| port.clone()),
            db_name: from.db_name.clone().or_else(|| db_name.clone()),
            user: from.user.clone().or_else(|| user.clone()),
            password: from.password.clone().or_else(|| password.clone()),
            connect_timeout_sec: from
                .connect_timeout_sec
                .or_else(|| connect_timeout_sec.clone()),
            is_node_online: None,
            postgres_is_leader: None,
            postgres_is_replica: None,
            citus_is_leader_coordinator_node: None,
            citus_is_replica_coordinator_node: None,
            citus_is_leader_worker_node: None,
            citus_is_replica_worker_node: None,
            citus_is_active_worker_node: None,
            citus_group_id: None,
            patroni_is_primary: None,
            patroni_is_replica: None,
            patroni_is_read_write: None,
            patroni_is_read_only: None,
        }
    }

    pub fn set_db_name(&mut self, db_name: String) {
        self.db_name = Some(db_name);
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "host={} port={} dbname={} user={} password={} connect_timeout={} application_name=taco",
            self.host,
            self.port.unwrap_or_default(),
            self.db_name.as_deref().unwrap_or("default_dbname"),
            self.user.as_deref().unwrap_or("default_user"),
            self.password.as_deref().unwrap_or("default_password"),
            self.connect_timeout_sec.unwrap_or_default()
        )
    }
}
