pub struct SettingsManager {
    _settings_file_name: String,
}

impl SettingsManager {
    pub fn new(settings_file_name: &str) -> Self {
        Self { _settings_file_name: settings_file_name.to_string() }
    }
}