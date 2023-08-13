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

//! Provides utility functions related to the environment setup and configuration.
//! This module contains functions to get the substrate directory, retrieve pallets,
//! and load hookpoint definitions from a configuration file.

use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::config::models::Definitions;

/// Returns the path to the Substrate directory.
///
/// If the `substrate_dir` option is `None`, it defaults to the current working directory.
///
/// # Examples
///
/// ```
/// let substrate_path = get_substrate_dir(&Some("/path/to/substrate"));
/// ```
pub(crate) fn get_substrate_dir<P: AsRef<Path>>(substrate_dir: &Option<P>) -> std::io::Result<PathBuf> {
   match substrate_dir {
        None => std::env::current_dir(),
        Some(ref substrate_dir) => Ok(PathBuf::from(substrate_dir.as_ref())),
    }
}

/// Returns a mapping of available pallet names to their respective paths.
///
/// # Examples
///
/// ```
/// let pallets = get_pallets(&Some("/path/to/substrate"));
/// ```
pub(crate) fn get_pallets<P: AsRef<Path>>(substrate_dir: &Option<P>) -> std::io::Result<HashMap<String, PathBuf>> {
    let dir = get_substrate_dir(substrate_dir)?;
    let mut dirs: HashMap<String, PathBuf> = HashMap::new();
    let pallets_dir = dir.join("pallets");
    for entry in fs::read_dir(pallets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            dirs.insert(path.file_name().unwrap().to_str().unwrap().to_string(), path);
        }
    }
    Ok(dirs)
}

/// Loads and parses the hookpoint definitions from `hookpoints.json`.
///
/// # Examples
///
/// ```
/// let definitions = load_definitions(&Some("/path/to/substrate"));
/// ```
pub(crate) fn load_definitions<P: AsRef<Path>>(substrate_dir: &Option<P>) -> std::io::Result<Definitions> {
    let dir = get_substrate_dir(substrate_dir)?;
    let config_path = dir.join("hookpoints.json");
    let content = fs::read_to_string(config_path)?;
    let definitions: Definitions = serde_json::from_str(&content)?;
    Ok(definitions)
}
