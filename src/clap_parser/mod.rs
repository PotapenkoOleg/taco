use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Taco database management tool")]
pub struct Args {
    #[arg(long, short, default_value = "inventory.taco.yml")]
    pub inventory: String,
}
