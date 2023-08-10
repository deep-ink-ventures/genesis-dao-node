mod config;
mod builder;
mod interactive;
mod environment;

#[cfg(test)]
mod tests;

use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use ctrlc;

use config::models::Definitions;
use builder::hooks::create_hooks;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure hookpoints.json interactively
    Configure {
       #[clap(short, long)]
       substrate_dir: Option<String>,
    },
    /// Generate hooks for hookpoints.json and ink! contract boilerplate
    Generate {
       #[clap(short, long)]
       substrate_dir: Option<String>,

        #[clap(short, long)]
        config: Option<String>,
    }
}

fn main() {

    ctrlc::set_handler(move || {
        println!("\nGoodbye! Checkout hookpoints.rs!\n");
    }).expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Configure { substrate_dir}) => {
            println!("\nWelcome to the hookpoint configuration wizard!");
            println!("\nYou can always abort the process by pressing Ctrl+C and manually change hookpoints.json.");

            let pallets = environment::get_pallets(substrate_dir).expect("Unable to load pallets from substrate directory");
            let name = interactive::set_name();
            let mut definitions = Definitions::new(
                name,
                pallets.keys().map(|pallet| (pallet.clone(), Vec::new())).collect(),
            );
            definitions.write_to_file();

            println!("\nI created a hookpoints.js file in your substrate root!\n");

            loop {
                let pallet_name = interactive::select_pallet(pallets.keys().cloned().collect());
                let pallet_function = interactive::add_hook();
                definitions.add_pallet_function(pallet_name, pallet_function);
                definitions.write_to_file();
                println!("\nSaved. Add another hook or end with CTRL+C.\n");
            }
        }

        Some(Commands::Generate { config, substrate_dir }) => {
            let config_file: String = match config {
                Some(config) => config.to_string(),
                None => "hookpoints.json".to_string()
            };
            let json_content = fs::read_to_string(config_file).expect("config not found, specify correct path with -c");
            let definitions: Definitions = serde_json::from_str(&json_content).expect("Failed to parse JSON definitions");
            let pallets = environment::get_pallets(substrate_dir).expect("Unable to load pallets from substrate directory");

            let hooks = create_hooks(definitions);
            // write hooks to file
            hooks.iter().for_each(|(name, content)| {
                let mut file = fs::File::create(format!("{}/src/hooks.rs", pallets.get(name).unwrap().display())).expect("Unable to create file");
                file.write_all(content.as_bytes()).expect("Unable to write file");
            });
        },

        None => ()
    }
}
