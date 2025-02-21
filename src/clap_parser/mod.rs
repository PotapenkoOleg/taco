use clap::Parser;

/// Command line arguments for the application
#[derive(Parser, Debug)]
#[command(author, version, about = "Taco database management tool")]
pub struct Args {
    /// Path to the inventory configuration file
    #[arg(long, short, default_value = "inventory.taco.yml")]
    pub inventory: String,

    /// Path to the settings configuration file
    #[arg(long, default_value = "settings.taco.yml")]
    pub settings: String,

    /// Path to the secrets configuration file
    #[arg(long, default_value = "secrets.taco.yml")]
    pub secrets: String,
}