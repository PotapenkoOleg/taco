use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, short, default_value = "inventory.taco.yml")]
    pub inventory: String,

    #[arg(long, default_value = "settings.taco.yml")]
    pub settings: String,

    #[arg(long, default_value = "secrets.taco.yml")]
    pub secrets: String,
}