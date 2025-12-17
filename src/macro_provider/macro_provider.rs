use crate::shared::request_type::RequestType;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub struct MacroProvider {
    macros: HashMap<&'static str, String>,
    macros_request_type: BTreeMap<&'static str, RequestType>,
    macros_request_description: HashMap<&'static str, String>,
}

impl MacroProvider {
    pub fn new() -> Self {
        let mut macros = HashMap::new();
        let mut macros_request_type = BTreeMap::new();
        let mut macros_request_description = HashMap::new();

        // region drop_db
        let drop_db = "\
        UPDATE pg_database SET datallowconn = false WHERE datname = '$DB_NAME$';
        SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='$DB_NAME$';
        DROP DATABASE $DB_NAME$;
        ";
        let drop_db_description = "CLOSE ALL CONNECTIONS AND DROPS DB $DB_NAME$";
        macros.insert("drop_db", drop_db.to_string());
        macros_request_type.insert("drop_db", RequestType::Command);
        macros_request_description.insert("drop_db", drop_db_description.to_string());
        // endregion

        // region pg_leader_status
        let pg_leader_status = "SELECT application_name, client_addr, client_port, state, sync_state FROM pg_stat_replication;";
        let pg_leader_status_description =
            "SHOWS POSTGRES LEADER REPLICATION STATUS FOR EVERY NODE IN GROUP";
        macros.insert("pg_leader_status", pg_leader_status.to_string());
        macros_request_type.insert("pg_leader_status", RequestType::Query);
        macros_request_description
            .insert("pg_leader_status", pg_leader_status_description.to_string());
        // endregion

        // region pg_replica_status
        let pg_replica_status =
            "SELECT status, last_msg_send_time, slot_name, sender_host FROM pg_stat_wal_receiver;";
        let pg_replica_status_description =
            "SHOWS POSTGRES REPLICA REPLICATION STATUS FOR EVERY NODE IN GROUP";
        macros.insert("pg_replica_status", pg_replica_status.to_string());
        macros_request_type.insert("pg_replica_status", RequestType::Query);
        macros_request_description.insert(
            "pg_replica_status",
            pg_replica_status_description.to_string(),
        );
        // endregion

        // region pg_replication_status
        let pg_replication_status = "\
        SELECT application_name, client_addr, client_port, state, sync_state FROM pg_stat_replication;\
        SELECT status, last_msg_send_time, slot_name, sender_host FROM pg_stat_wal_receiver;\
        ";
        let pg_replication_status_description =
            "SHOWS ALL POSTGRES REPLICATION STATUS FOR EVERY NODE IN GROUP";
        macros.insert("pg_replication_status", pg_replication_status.to_string());
        macros_request_type.insert("pg_replication_status", RequestType::Query);
        macros_request_description.insert(
            "pg_replication_status",
            pg_replication_status_description.to_string(),
        );
        // endregion

        Self {
            macros,
            macros_request_type,
            macros_request_description,
        }
    }

    pub fn is_macro_exists(&self, macro_name: &str) -> bool {
        self.macros.contains_key(macro_name)
    }

    pub fn get_macro_names(&self) -> Vec<(String, String)> {
        self.macros_request_type
            .keys()
            .map(|k| (k.to_string(), self.macros_request_description[k].clone()))
            .collect::<Vec<(String, String)>>()
    }

    pub fn get_macro_parameters(&self, name: &str) -> Option<Vec<String>> {
        if let Some(macros) = self.macros.get(name) {
            let char_to_find = '$';

            let found_indexes: Vec<usize> = macros
                .char_indices()
                .filter(|&(_, c)| c == char_to_find)
                .map(|(i, _)| i)
                .collect();
            if found_indexes.is_empty() || found_indexes.len() % 2 != 0 {
                return None;
            }
            let mut macro_parameters = BTreeSet::<String>::new(); // use for order
            for i in found_indexes.chunks(2) {
                let start = i[0];
                let end = i[1];
                let param = macros.get(start..end + 1);
                if let Some(param) = param {
                    macro_parameters.insert(param.to_string());
                }
            }
            if macro_parameters.is_empty() {
                return None;
            }
            let result: Vec<String> = macro_parameters.iter().map(|s| s.to_string()).collect();
            return Some(result);
        }
        None
    }
    pub fn get_macro(
        &self,
        name: &str,
        macro_parameters: Option<Vec<String>>,
        macro_values: HashMap<String, String>,
    ) -> Option<Vec<String>> {
        if let Some(macros) = self.macros.get(name) {
            let mut new_macros = macros.clone();
            if let Some(macro_parameters) = macro_parameters {
                for param in macro_parameters {
                    new_macros = new_macros.replace(&param, &macro_values[&param]);
                }
            }
            new_macros = new_macros.trim().to_string();
            let parts = new_macros.split(';');
            let mut commands: Vec<String> = parts
                .into_iter()
                .map(|x| x.to_string().trim().to_string())
                .collect();
            commands.remove(commands.len() - 1);
            let commands_mut_ref = &mut commands;
            commands_mut_ref.into_iter().for_each(|x| x.push(';'));

            return Some(commands);
        }
        None
    }

    pub fn get_macro_request_type(&self, request_type: &str) -> Option<RequestType> {
        self.macros_request_type.get(request_type).cloned()
    }
}
