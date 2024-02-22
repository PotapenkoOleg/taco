pub struct SecretsManager {
    secrets_file_name: String,
}

impl SecretsManager {
    pub fn new(secrets_file_name: &str) -> Self {
        Self { secrets_file_name: secrets_file_name.to_string() }
    }
}