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

//! Provides interactive prompts to help set up a new substrate hookpoint project.
//! This module contains functions that gather project details, such as names,
//! hook points, argument details, and return types, through user input.

extern crate dialoguer;
extern crate regex;

use dialoguer::{theme::ColorfulTheme, Select, Input};
use crate::config::models::{FunctionArgument, PalletFunction, ReturnValue};
use crate::utils::INK_TYPES;

pub const SNAKE_CASE_REGEX: &str = r"^[a-z]+(?:_[a-z]+)*$";
pub const CAMEL_CASE_REGEX: &str = r"^[A-Z][a-z]+(?:[A-Z][a-z]+)*$";

/// Prompts the user for a project name in CamelCase format.
///
/// # Examples
///
/// ```
/// let project_name = set_name();
/// ```
pub fn set_name() -> String {
    println!("\nEnter a name for your project (CamelCase please):\n");
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Name (CamelCase please):")
        .validate_with({
            let camel_case_regex = regex::Regex::new(CAMEL_CASE_REGEX).unwrap();
            move |input: &String| -> Result<(), &str> {
                if camel_case_regex.is_match(input) {
                    Ok(())
                } else {
                    Err("This is not a valid name; please use CamelCase")
                }
            }
        })
        .interact_text()
        .unwrap()
}

/// Allows the user to select a pallet from the provided list.
///
/// # Examples
///
/// ```
/// let available_pallets = vec!["pallet1".to_string(), "pallet2".to_string()];
/// let selected_pallet = select_pallet(available_pallets);
/// ```
pub fn select_pallet(pallets: Vec<String>) -> String {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a pallet to hook into:")
        .default(0)
        .items(&pallets)
        .interact()
        .unwrap();

    pallets[selection].to_string()
}

/// Collects a hook point name in snake_case format from the user.
///
/// # Examples
///
/// ```
/// let hook_point_name = get_hook_point_name();
/// ```
pub fn get_hook_point_name() -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter a hook point name (snake_case please):")
        .validate_with({
            let snake_case_regex = regex::Regex::new(SNAKE_CASE_REGEX).unwrap();
            move |input: &String| -> Result<(), &str> {
                if snake_case_regex.is_match(input) {
                    Ok(())
                } else {
                    Err("This is not a valid hook point name; please use snake_case")
                }
            }
        }).interact_text().unwrap()
}

/// Prompts the user for argument details, including name and type.
///
/// # Examples
///
/// ```
/// let ink_types = vec!["type1", "type2"];
/// let argument = get_argument_details(&ink_types);
/// ```
pub fn get_argument_details(ink_types: &Vec<&str>) -> FunctionArgument {
    let arg_name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter an argument name (snake_case please):")
        .validate_with({
            let snake_case_regex = regex::Regex::new(SNAKE_CASE_REGEX).unwrap();
            move |input: &String| -> Result<(), &str> {
                if snake_case_regex.is_match(input) {
                    Ok(())
                } else {
                    Err("This is not a valid argument name; please use snake_case")
                }
            }
        }).interact_text().unwrap();

    let arg_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a type:")
        .default(0)
        .items(ink_types)
        .interact()
        .unwrap();

    FunctionArgument {
        name: arg_name,
        type_: ink_types[arg_type].to_string(),
    }
}

/// Obtains details about the return type of a hook.
///
/// # Examples
///
/// ```
/// let ink_types = vec!["type1", "type2"];
/// let arguments = vec![FunctionArgument { name: "arg1".to_string(), type_: "type1".to_string() }];
/// let no_default_values = vec!["type1"];
/// let return_details = get_return_details(&ink_types, &arguments, &no_default_values);
/// ```
pub fn get_return_details(ink_types: &Vec<&str>, arguments: &Vec<FunctionArgument>, no_default_values: &Vec<&str>) -> ReturnValue {
    let return_type = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a return type:")
        .default(0)
        .items(ink_types)
        .interact()
        .unwrap();

    let mut arguments_with_return_type: Vec<&str> = arguments.iter()
        .filter(|arg| arg.type_ == ink_types[return_type])
        .filter(|arg| !no_default_values.contains(&arg.type_.as_str()))
        .map(|arg| arg.name.as_str())
        .collect();
    arguments_with_return_type.push(ink_types[return_type]);

    let return_default = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a default return value:")
        .default(0)
        .items(&arguments_with_return_type)
        .interact()
        .unwrap();

    ReturnValue {
        type_: ink_types[return_type].to_string(),
        default: arguments_with_return_type[return_default].to_string(),
    }
}

/// Facilitates the creation of a new hook by gathering necessary details from the user.
///
/// # Examples
///
/// ```
/// let new_hook = add_hook();
/// ```
pub fn add_hook() -> PalletFunction {
    let hook_point = get_hook_point_name();

    let mut arguments: Vec<FunctionArgument> = Vec::new();
    let mut returns: Option<ReturnValue> = None;
    let ink_types = INK_TYPES.to_vec();
    let no_default_values = vec!["AccountId"];

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose an option:")
            .default(0)
            .items(&["Add a function argument", "Set a return value", "Save & create another hook ..."])
            .interact()
            .unwrap();

        match selection {
            0 => {
                let arg = get_argument_details(&ink_types);
                arguments.push(arg);
            }
            1 => {
                let ret = get_return_details(&ink_types, &arguments, &no_default_values);
                returns = Some(ret);
            }
            _ => break
        }
    }

    PalletFunction {
        hook_point,
        arguments,
        returns,
    }
}
