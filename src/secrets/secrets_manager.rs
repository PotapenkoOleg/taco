pub struct SecretsManager {
    _secrets_file_name: String,
}

impl SecretsManager {
    pub fn new(secrets_file_name: &str) -> Self {
        Self { _secrets_file_name: secrets_file_name.to_string() }
    }
}