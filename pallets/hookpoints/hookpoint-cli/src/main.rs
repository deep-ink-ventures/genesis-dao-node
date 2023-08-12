mod config;
mod builder;
mod interactive;
mod environment;

#[cfg(test)]
mod tests;
mod utils;

use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use ctrlc;

use config::models::Definitions;
use builder::hooks::create_hooks;
use crate::builder::contracts::{generate_contract_boilerplate_toml, generate_contract_trait_toml, generate_ink_boilerplate_contract, generate_ink_trait};
use crate::environment::get_substrate_dir;
use crate::utils::camel_case_to_kebab;

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
    },
}

fn main() {
    ctrlc::set_handler(move || {
        println!("\nGoodbye! Checkout hookpoints.rs!\n");
    }).expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Configure { substrate_dir }) => {
            println!("\nWelcome to the hookpoint configuration wizard!");
            println!("\nYou can always abort the process by pressing Ctrl+C and manually change hookpoints.json.");

            let definition_result = environment::load_definitions(substrate_dir);
            let pallets = environment::get_pallets(substrate_dir).expect("Unable to load pallets from substrate directory");

            let mut definitions = match { definition_result } {
                Ok(definitions) => definitions,
                Err(_) => {
                    let name = interactive::set_name();
                    println!("\nI am creating a hookpoints.js file in your substrate root!\n");
                    Definitions::new(
                        name,
                        pallets.keys().map(|pallet| (pallet.clone(), Vec::new())).collect(),
                    )
                }
            };

            loop {
                let pallet_name = interactive::select_pallet(pallets.keys().cloned().collect());
                let pallet_function = interactive::add_hook();
                definitions.add_pallet_function(pallet_name, pallet_function);
                definitions.write_to_file(substrate_dir);
                println!("\nSaved. Add another hook or end with CTRL+C.\n");
            }
        }

        Some(Commands::Generate { config, substrate_dir }) => {
            let root = get_substrate_dir(substrate_dir).expect("Unable to get root dir");

            let config_file: String = match config {
                Some(config) => config.to_string(),
                None => {
                    let mut config_path = root.clone();
                    config_path.push("hookpoints.json");
                    config_path.to_str().map(|s| s.to_string()).unwrap()
                }
            };
            println!("Using config file: {}", config_file);
            let json_content = fs::read_to_string(config_file).expect("config not found, specify correct path with -c");
            let definitions: Definitions = serde_json::from_str(&json_content).expect("Failed to parse JSON definitions");
            let pallets = environment::get_pallets(substrate_dir).expect("Unable to load pallets from substrate directory");

            let hooks = create_hooks(definitions.clone());
            // write hooks to file
            hooks.iter().for_each(|(name, content)| {
                let mut file = fs::File::create(format!("{}/src/hooks.rs", pallets.get(name).unwrap().display())).expect("Unable to create file");
                file.write_all(content.as_bytes()).expect("Unable to write file");
            });


            // Create trait ...
            let path = root.join(format!("contracts/hooks/{}-contract-trait", camel_case_to_kebab(&definitions.name)));
            if !path.exists() {
                fs::create_dir_all(&path).expect("Unable to create trait folder");
            }

            // Write to Cargo.toml
            let trait_toml = generate_contract_trait_toml(&definitions).expect("invalid toml generated");
            let mut trait_toml_file = fs::File::create(path.join("Cargo.toml")).expect("Unable to create toml file");
            trait_toml_file.write_all(trait_toml.as_bytes()).expect("Unable to write toml file");

            // Write to lib.rs
            let trait_content = generate_ink_trait(&definitions);
            let mut lib_file = fs::File::create(path.join("lib.rs")).expect("Unable to create the traits lib.rs");
            lib_file.write_all(trait_content.as_bytes()).expect("unable to write the traits lib.rs");

            // Create boilerplate
            let path = root.join(format!("contracts/hooks/{}-contract-boilerplate", camel_case_to_kebab(&definitions.name)));
            if !path.exists() {
                fs::create_dir_all(&path).expect("Unable to create boilerplate folder");
            }
            let test_path = root.join(format!("contracts/hooks/{}-contract-tests", camel_case_to_kebab(&definitions.name)));
            if !test_path.exists() {
                fs::create_dir_all(&test_path).expect("Unable to create tests folder");
            }

            // Write to Cargo.toml
            let boilerplate_toml = generate_contract_boilerplate_toml(&definitions).expect("invalid toml generated");
            let mut boilerplate_toml_file = fs::File::create(path.join("Cargo.toml")).expect("Unable to create toml file");
            boilerplate_toml_file.write_all(boilerplate_toml.as_bytes()).expect("Unable to write toml file");

            let mut test_toml_file = fs::File::create(test_path.join("Cargo.toml")).expect("Unable to create toml file");
            test_toml_file.write_all(boilerplate_toml.as_bytes()).expect("Unable to write toml file");

            // Write to lib.rs
            let contract_content = generate_ink_boilerplate_contract(&definitions);
            let mut lib_file = fs::File::create(path.join("lib.rs")).expect("Unable to create the contract lib.rs");
            lib_file.write_all(contract_content.as_bytes()).expect("unable to write the contract lib.rs");

            let mut test_lib_file = fs::File::create(test_path.join("lib.rs")).expect("Unable to create the contract lib.rs");
            test_lib_file.write_all(contract_content.as_bytes()).expect("unable to write the contract lib.rs");
        }

        None => ()
    }
}
