use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SettingsProvider {
    settings: Arc<Mutex<HashMap<String, String>>>,
}

impl SettingsProvider {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(HashMap::<String, String>::new())),
        }
    }
    pub fn get_key(&self, key: String) -> Option<String> {
        let settings_lock = self.settings.lock().unwrap();
        match settings_lock.get(&key) {
            Some(value) => Some(value.clone()),
            _ => None,
        }
    }
    pub fn set_key(&mut self, key: String, value: String) {
        let mut settings_lock = self.settings.lock().unwrap();
        settings_lock.insert(key, value);
    }
}
