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
    #[serde(skip_serializing)]
    pub is_active: bool,
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
            is_active: true,
        }
    }

    pub fn set_db_name(&mut self, db_name: String) {
        self.db_name = Some(db_name);
    }
    pub fn get_db_name(&self) -> Option<String> {
        self.db_name.clone()
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


