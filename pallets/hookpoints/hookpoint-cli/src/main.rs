mod config;
mod builder;
pub(crate) mod substrate;
mod interactive;
mod utils;

use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use ctrlc;

use config::models::Definitions;
use builder::hooks::create_hooks;
use substrate::Substrate;

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
        config: Option<String>,
    }
}

fn main() {

    ctrlc::set_handler(move || {
        println!("\nGoodbye! Checkout hookpoints.rs!\n");
    }).expect("Error setting Ctrl-C handler");

    let substrate = Substrate::new(Some(&String::from("/Users/chp/Projects/deep-ink/genesis-dao/node")));
    let cli = Cli::parse();

    match &cli.command {
        None => {
            println!("\nWelcome to the hookpoint configuration wizard!");
            println!("\nYou can always abort the process by pressing Ctrl+C and manually change hookpoints.json.");
            let name = interactive::set_name();
            let mut definitions = Definitions::new(
                name,
                substrate.pallets.keys().map(|pallet| (pallet.clone(), Vec::new())).collect(),
            );
            definitions.write_to_file();

            println!("\n\n");
            let pallet_name = interactive::select_pallet(substrate.pallets.keys().cloned().collect());
            let pallet_function = interactive::add_hook();

            definitions.add_pallet_function(pallet_name, pallet_function);
            definitions.write_to_file();

        }

        Some(Commands::Generate { config }) => {
            let config_file: String = match config {
                Some(config) => config.to_string(),
                None => "hookpoints.json".to_string()
            };
            let json_content = fs::read_to_string(config_file).expect("config not found, specify correct path with -c");
            let definitions: Definitions = serde_json::from_str(&json_content).expect("Failed to parse JSON definitions");

            let hooks = create_hooks(definitions);
            // write hooks to file
            hooks.iter().for_each(|(name, content)| {
                let mut file = fs::File::create(format!("{}/src/hooks.rs", substrate.pallets.get(name).unwrap().display())).expect("Unable to create file");
                file.write_all(content.as_bytes()).expect("Unable to write file");
            });
        }
    }
}
