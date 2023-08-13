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

//! # Hooks Builder
//!
//! This module provides functionality to generate the required hooks based on the provided
//! configuration from `hookpoints.json`. The generated hooks act as the interface between
//! the Substrate runtime and the ink! smart contracts.
//!
//! The main function to use is `create_hooks`, which takes the `Definitions` structure
//! from the configuration and returns a mapping of pallet names to the generated hooks' content.

use std::collections::HashMap;
use crate::builder::mapper::ink_to_substrate;
use crate::config::models::{Definitions, ReturnValue};

/// Generates a function signature for the given function name, arguments, and optional return type.
///
/// # Arguments
///
/// * `name` - The name of the function.
/// * `arguments` - A slice of tuples containing argument names and their types.
/// * `returns` - An optional `ReturnValue` that specifies the return type and default value.
///
/// # Returns
///
/// Returns a string representing the generated function signature.
fn generate_function_signature(name: &str, arguments: &[(String, String)], returns: &Option<ReturnValue>) -> String {
    let mut args = arguments.iter()
        .map(|(arg_name, arg_type)| format!("{}: {}", arg_name, ink_to_substrate(arg_type)))
        .collect::<Vec<String>>()
        .join(", ");
    if arguments.len() > 0 {
        args.insert_str(0, ", ");
    }

    let mut func_sig = format!("pub fn {}<T: Config>(owner: T::AccountId, signer: T::AccountId{})", name, args);
    if let Some(r) = returns {
        func_sig.push_str(format!(" -> {}", ink_to_substrate(r.type_.as_str())).as_str());
    }
    func_sig

}

/// Generates the body of a function for the provided name, hook point, arguments, and optional return type.
///
/// # Arguments
///
/// * `name` - The name of the function.
/// * `hook_point` - The specific point where the hook should be executed.
/// * `arguments` - A slice of tuples containing argument names and their types.
/// * `returns` - An optional `ReturnValue` that specifies the return type and default value.
///
/// # Returns
///
/// Returns a string representing the generated function body.
fn generate_function_body(name: &str, hook_point: &str, arguments: &[(String, String)], returns: &Option<ReturnValue>) -> String {
   let hp_def = format!(r#"
   let hp = HP::<T>::create(
		"{}::{}",
		owner,
		signer
	)"#, name, hook_point);

    let mut args = arguments.iter()
        .map(|(arg_name, arg_type)| format!("\n\t\t.add_arg::<{}>({})", ink_to_substrate(arg_type), arg_name))
        .collect::<Vec<String>>()
        .join("");
    if arguments.len() > 0 {
        args.insert_str(0, " ");
    }
    args.push_str(";");

    let execute = match returns {
        None => String::from("\n\n\tHP::<T>::execute::<()>(hp)"),
        Some(r) => format!("\n\n\tHP::<T>::execute::<{}>(hp).unwrap_or({})", ink_to_substrate(r.type_.as_str()), r.default)
    };

    format!("{}{}{}", hp_def, args, execute)
}

/// Generates hooks for the provided configuration and returns a mapping from pallet names to generated hooks' content.
///
/// # Arguments
///
/// * `config` - The `Definitions` structure that contains the configuration for generating the hooks.
///
/// # Returns
///
/// Returns a `HashMap` where the keys are pallet names and the values are strings containing the content of the generated hooks.
///
/// # Examples
///
/// ```
/// let config = Definitions { /*... initialization ...*/ };
/// let hooks = create_hooks(config);
/// // Now, `hooks` contains a mapping of pallet names to generated hooks.
/// ```
pub fn create_hooks(config: Definitions) -> HashMap<String, String> {
    let mut pallet_to_hooks: HashMap<String, String> = HashMap::new();

    for (pallet_name, pallet_functions) in config.pallets {
        let mut funcs: Vec<String> = vec![];
        for pallet_function in pallet_functions {
            let args = pallet_function.arguments.iter().map(|arg| (arg.name.clone(), arg.type_.clone())).collect::<Vec<(String, String)>>();
            let function_signature = generate_function_signature(
                &pallet_function.hook_point,
                &args,
                &pallet_function.returns
            );

            let function_body = generate_function_body(
                &config.name,
                &pallet_function.hook_point,
                &args,
                &pallet_function.returns
            );

            funcs.push(format!("{}\n{{ {}\n}}", function_signature, function_body));
        }

        if funcs.len() > 0 {
            let mut content = String::from("use crate::Config;\nuse pallet_hookpoints::Pallet as HP;\n\n");
            content.push_str(&funcs.join("\n\n"));
            pallet_to_hooks.insert(pallet_name, content);
        }
    }
    pallet_to_hooks
}
