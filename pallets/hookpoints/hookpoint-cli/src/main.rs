mod config;
mod builder;
mod environment;

use clap::{Parser, Subcommand};
use std::{env, fs};

use config::models::Configuration;
use builder::hooks::create_hooks;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[clap(short, long)]
        config: String,
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Generate { config, root }) => {
            let json_content = fs::read_to_string(config).expect("JSON config required");
            let config: Configuration = serde_json::from_str(&json_content).expect("Failed to parse JSON config");


            println!("{}", env::current_dir().unwrap().to_str().unwrap());
            create_hooks(config);
        }
        None => {}
    }
}
