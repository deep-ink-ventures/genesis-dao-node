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

//! # Hookpoint Configuration Models
//!
//! This module defines the primary data structures and their associated methods
//! that are used to represent and manipulate the configuration for hookpoints.
//!
//! The primary structures are:
//! - `Definitions`: Represents the overall configuration, including the project name, associated ink dependencies, and the set of pallets.
//! - `InkDependencies`: Contains the version details for ink and related dependencies.
//! - `PalletFunction`: Represents an individual function within a pallet that can be targeted by hookpoints.
//! - `FunctionArgument`: Represents an argument of a `PalletFunction`.
//! - `ReturnValue`: Represents the return value of a `PalletFunction`.
//!
//! The module also provides various utility methods that allow for reading, writing, and manipulating the configuration data.

use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

/// Represents the overall configuration of hookpoints, encompassing details such as the
/// project name, associated ink dependencies, and the set of pallets that are to be hooked.
///
/// # Examples
///
/// ```
/// let defs = Definitions::new("MyProject", my_pallets_map);
/// defs.write_to_file(None);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Definitions {
    pub name: String,
    pub ink_dependencies: InkDependencies,
    pub pallets: std::collections::HashMap<String, Vec<PalletFunction>>,
}


/// Contains version details of ink-related dependencies.
///
/// # Examples
///
/// ```
/// let ink_deps = InkDependencies::default();
/// println!("Ink Version: {}", ink_deps.ink_version);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InkDependencies {
    pub ink_version: String,
    pub ink_primitives_version: String,
    pub scale_version: String,
    pub scale_info_version: String,
}

/// Represents an individual function within a pallet that hookpoints can target.
///
/// # Examples
///
/// ```
/// let func_arg = FunctionArgument { name: "arg1".to_string(), type_: "u32".to_string() };
/// let ret_val = ReturnValue { default: "0".to_string(), type_: "u32".to_string() };
/// let pallet_func = PalletFunction { hook_point: "some_hook".to_string(), arguments: vec![func_arg], returns: Some(ret_val) };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PalletFunction {
    pub hook_point: String,
    pub arguments: Vec<FunctionArgument>,
    pub returns: Option<ReturnValue>,
}

/// Represents an argument within a `PalletFunction`. Contains the name of the argument and its associated type.
///
/// # Examples
///
/// ```
/// let func_arg = FunctionArgument { name: "arg1".to_string(), type_: "u32".to_string() };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

/// Represents a return value within a `PalletFunction`. Contains the default value of the return and its associated type.
///
/// # Examples
///
/// ```
/// let ret_val = ReturnValue { default: "0".to_string(), type_: "u32".to_string() };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReturnValue {
    pub default: String,
    #[serde(rename = "type")]
    pub type_: String,
}


impl Definitions {
    /// Creates a new `Definitions` object with a given name and pallets.
    ///
    /// # Examples
    ///
    /// ```
    /// let defs = Definitions::new("MyProject", my_pallets_map);
    /// ```
    pub fn new(name: String, pallets: std::collections::HashMap<String, Vec<PalletFunction>>) -> Self {
        Definitions {
            name,
            pallets,
            ink_dependencies: InkDependencies::default(),
        }
    }

    /// Writes the current state of the `Definitions` object to a file.
    ///
    /// # Examples
    ///
    /// ```
    /// defs.write_to_file(None);
    /// ```
    pub fn write_to_file<P: AsRef<Path>>(&self, substrate_dir: &Option<P>) {
        let dir = match substrate_dir {
            None => std::env::current_dir().unwrap(),
            Some(dir) => PathBuf::from(dir.as_ref()),
        };
        let config_path = dir.join("hookpoints.json");
        let content = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(config_path, content).unwrap();
    }

    /// Adds a `PalletFunction` to the current set of functions for a given pallet name.
    ///
    /// # Examples
    ///
    /// ```
    /// let func = PalletFunction { /* ... */ };
    /// defs.add_pallet_function("my_pallet", func);
    /// ```
    pub fn add_pallet_function(&mut self, pallet_name: String, pallet_function: PalletFunction) {
        if let Some(pallet_functions) = self.pallets.get_mut(&pallet_name) {
            pallet_functions.push(pallet_function);
        } else {
            self.pallets.insert(pallet_name, vec![pallet_function]);
        }
    }

    /// Extracts all types from the current set of pallet functions.
    ///
    /// # Examples
    ///
    /// ```
    /// let types = defs.extract_types();
    /// ```
    pub fn extract_types(&self) -> Vec<String> {
        let mut types = Vec::new();

        for functions in self.pallets.values() {
            for func in functions {
                for arg in &func.arguments {
                    types.push(arg.type_.clone());
                }

                if let Some(ret_val) = &func.returns {
                    types.push(ret_val.type_.clone());
                }
            }
        }

        types
    }

    /// Checks if the current set of pallet functions contains a specific type.
    ///
    /// # Examples
    ///
    /// ```
    /// let contains = defs.contains_type(&["u32", "String"]);
    /// ```
    pub fn contains_type(&self, target: &[&str]) -> bool {
        let types = self.extract_types();
        for t in types {
            if target.contains(&t.as_str()) {
                return true;
            }
        }
        false
    }
}

impl InkDependencies {
    /// Creates a default `InkDependencies` object with pre-defined version numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// let ink_deps = InkDependencies::default();
    /// ```
    pub fn default() -> Self {
        InkDependencies {
            ink_version: "4.2".to_string(),
            ink_primitives_version: "4.2".to_string(),
            scale_version: "3".to_string(),
            scale_info_version: "2.6".to_string(),
        }
    }
}
