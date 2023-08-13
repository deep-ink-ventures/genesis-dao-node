// Copyright (C) Deep Ink Ventures GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Hookpoint CLI: A Comprehensive Tool for Substrate Runtimes
//!
//! The Hookpoint CLI is designed to facilitate seamless interactions with Substrate runtimes,
//! enabling developers to configure and generate hookpoints, which are pivotal integration points
//! within a runtime's lifecycle. By offering an interactive configuration wizard, the CLI empowers
//! developers to define and manage `hookpoints.json`, which acts as the blueprint for these integration points.
//!
//! The CLI also offers robust capabilities to generate hooks based on the provided configurations.
//! These hooks can be seamlessly integrated into the Substrate runtime, ensuring that custom logic
//! can be executed at the specified hookpoints. Furthermore, the tool provides mechanisms to generate
//! boilerplate contracts for the ink! smart contract platform. These contracts serve as templates,
//! simplifying the process of deploying custom logic on-chain.

//! The main entry point for the Hookpoint CLI.
//!
//! This application facilitates the configuration of hookpoints, the generation of hooks,
//! and the establishment of boilerplate contracts for the Substrate runtime.

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

/// Represents the command-line structure, defining the available commands and their parameters.
#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Enumerates the possible subcommands: Configure and Generate.
#[derive(Subcommand)]
enum Commands {
    /// Command to interactively configure the `hookpoints.json` file.
    Configure {
        #[clap(short, long)]
        substrate_dir: Option<String>,
    },
    /// Command to generate hooks based on the `hookpoints.json` and produce ink! contract boilerplate.
    Generate {
        #[clap(short, long)]
        substrate_dir: Option<String>,
        #[clap(short, long)]
        config: Option<String>,
    },
}

/// The main function that drives the Hookpoint CLI.
/// It handles command-line arguments, manages interactive sessions, and coordinates generation tasks.
fn main() {
    ctrlc::set_handler(move || {
        println!("\n[Hookpoint CLI] Process terminated gracefully. Thank you for using Hookpoints!");
    }).expect("Error setting Ctrl-C handler");

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Configure { substrate_dir }) => {
            println!("\n[Hookpoint CLI] Initiating the configuration wizard...");
            println!("\nTip: Use Ctrl+C at any time to terminate or make manual adjustments to `hookpoints.json`.");

            let definition_result = environment::load_definitions(substrate_dir);
            let pallets = environment::get_pallets(substrate_dir).expect("Unable to fetch pallets from substrate directory");

            let mut definitions = match definition_result {
                Ok(definitions) => definitions,
                Err(_) => {
                    let name = interactive::set_name();
                    println!("\n[Hookpoint CLI] Generating a `hookpoints.json` in your substrate root directory.");
                    Definitions::new(name, pallets.keys().map(|pallet| (pallet.clone(), Vec::new())).collect())
                }
            };

            loop {
                let pallet_name = interactive::select_pallet(pallets.keys().cloned().collect());
                let pallet_function = interactive::add_hook();
                definitions.add_pallet_function(pallet_name, pallet_function);
                definitions.write_to_file(substrate_dir);
                println!("\n[Hookpoint CLI] Changes saved. Add another hook or exit with Ctrl+C.");
            }
        }

        Some(Commands::Generate { config, substrate_dir }) => {
            let root = get_substrate_dir(substrate_dir).expect("Unable to determine root directory");

            let config_file: String = match config {
                Some(config) => config.to_string(),
                None => {
                    let mut config_path = root.clone();
                    config_path.push("hookpoints.json");
                    config_path.to_str().map(|s| s.to_string()).unwrap()
                }
            };
            println!("[Hookpoint CLI] Using configuration file at: {}", config_file);
            let json_content = fs::read_to_string(config_file).expect("Configuration not found; specify the correct path with -c");
            let definitions: Definitions = serde_json::from_str(&json_content).expect("Failed to interpret JSON definitions");
            let pallets = environment::get_pallets(substrate_dir).expect("Unable to load pallets from substrate directory");

            let hooks = create_hooks(definitions.clone());
            hooks.iter().for_each(|(name, content)| {
                let mut file = fs::File::create(format!("{}/src/hooks.rs", pallets.get(name).unwrap().display())).expect("File creation failed");
                file.write_all(content.as_bytes()).expect("File write operation failed");
            });

            let path = root.join(format!("contracts/hooks/{}-contract-trait", camel_case_to_kebab(&definitions.name)));
            if !path.exists() {
                fs::create_dir_all(&path).expect("Trait folder creation failed");
            }

            let trait_toml = generate_contract_trait_toml(&definitions).expect("TOML generation failed");
            let mut trait_toml_file = fs::File::create(path.join("Cargo.toml")).expect("TOML file creation failed");
            trait_toml_file.write_all(trait_toml.as_bytes()).expect("TOML file write failed");

            let trait_content = generate_ink_trait(&definitions);
            let mut lib_file = fs::File::create(path.join("lib.rs")).expect("lib.rs file creation for trait failed");
            lib_file.write_all(trait_content.as_bytes()).expect("lib.rs file write for trait failed");

            let path = root.join(format!("contracts/hooks/{}-contract-boilerplate", camel_case_to_kebab(&definitions.name)));
            if !path.exists() {
                fs::create_dir_all(&path).expect("Boilerplate folder creation failed");
            }
            let test_path = root.join(format!("contracts/hooks/{}-contract-tests", camel_case_to_kebab(&definitions.name)));
            if !test_path.exists() {
                fs::create_dir_all(&test_path).expect("Test folder creation failed");
            }

            let boilerplate_toml = generate_contract_boilerplate_toml(&definitions).expect("TOML generation for boilerplate failed");
            let mut boilerplate_toml_file = fs::File::create(path.join("Cargo.toml")).expect("TOML file creation for boilerplate failed");
            boilerplate_toml_file.write_all(boilerplate_toml.as_bytes()).expect("TOML file write for boilerplate failed");

            let mut test_toml_file = fs::File::create(test_path.join("Cargo.toml")).expect("Test TOML file creation failed");
            test_toml_file.write_all(boilerplate_toml.as_bytes()).expect("Test TOML file write failed");

            let contract_content = generate_ink_boilerplate_contract(&definitions);
            let mut lib_file = fs::File::create(path.join("lib.rs")).expect("lib.rs file creation for boilerplate failed");
            lib_file.write_all(contract_content.as_bytes()).expect("lib.rs file write for boilerplate failed");

            let mut test_lib_file = fs::File::create(test_path.join("lib.rs")).expect("Test lib.rs file creation failed");
            test_lib_file.write_all(contract_content.as_bytes()).expect("Test lib.rs file write failed");
        }

        None => ()
    }
}
