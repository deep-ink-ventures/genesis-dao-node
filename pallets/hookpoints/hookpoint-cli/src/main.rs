mod config;
mod builder;
pub(crate) mod substrate;
mod interactive;

use clap::{Parser, Subcommand};
use std::{env, fs};
use std::io::Write;
use ctrlc;

use config::models::Definitions;
use builder::hooks::create_hooks;
use substrate::Substrate;
use crate::interactive::select_pallet;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    New {},
    Generate {
        #[clap(short, long)]
        config: String,
    }
}

fn main() {

    ctrlc::set_handler(move || {
        println!("\nGoodbye! Checkout hookpoints.rs!\n");
    }).expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::New {}) => {
            let substrate = Substrate::new(Some(&String::from("/home/chp/projects/deep-ink/genesis/genesis-dao-node")));

            println!("\nWelcome to the hookpoint configuration wizard!");
            println!("\nYou can always abort the process by pressing Ctrl+C and manually change hookpoints.json.");
            println!("\nLet's start to configure your hookpoints!");

            let pallet_name = select_pallet(substrate.pallets.keys().cloned().collect());
        }
        Some(Commands::Generate { config }) => {
            let json_content = fs::read_to_string(config).expect("JSON config required");
            let definitions: Definitions = serde_json::from_str(&json_content).expect("Failed to parse JSON definitions");
            let substrate = Substrate::new(Some(&definitions.config.root_folder));

            let hooks = create_hooks(definitions);
            // write hooks to file
            hooks.iter().for_each(|(name, content)| {
                let mut file = fs::File::create(format!("{}/src/hooks.rs", substrate.pallets.get(name).unwrap().display())).expect("Unable to create file");
                file.write_all(content.as_bytes()).expect("Unable to write file");
            });
        }
        None => {}
    }
}
