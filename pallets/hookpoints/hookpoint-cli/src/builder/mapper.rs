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

//! # Type Mapping between ink! and Substrate
//!
//! This module provides utilities for mapping between ink! types and their corresponding
//! Substrate types. The mapping is essential to bridge the gap between contract-specific
//! types and the types used within the Substrate runtime.
//!
//! The primary functionality is exposed through the `ink_to_substrate` function, which
//! takes an ink! type and returns its Substrate counterpart.

use std::collections::HashMap;

/// Initializes a mapping of ink! types to their corresponding Substrate types.
///
/// # Returns
///
/// Returns a `HashMap` where the keys are ink! type strings and the values are the corresponding
/// Substrate type strings.
fn initialize_type_mapper() -> HashMap<String, String> {
    let mut mapper = HashMap::new();

    // Specialized types for substrate/ink
    mapper.insert("Balance".to_string(), "T::Balance".to_string());
    mapper.insert("AccountId".to_string(), "T::AccountId".to_string());

    mapper
}

/// Maps an ink! type to its corresponding Substrate type using a provided mapper.
///
/// # Arguments
///
/// * `mapper` - A reference to a `HashMap` that defines the mapping between ink! and Substrate types.
/// * `type_str` - The ink! type string to be mapped.
///
/// # Returns
///
/// Returns a string representing the mapped Substrate type. If no mapping is found, the original type string is returned.
fn map_type(mapper: &HashMap<String, String>, type_str: &str) -> String {
    mapper.get(type_str).cloned().unwrap_or_else(|| type_str.to_string())
}

/// Maps an ink! type to its corresponding Substrate type.
///
/// # Arguments
///
/// * `type_str` - The ink! type string to be mapped.
///
/// # Returns
///
/// Returns a string representing the mapped Substrate type.
///
/// # Examples
///
/// ```
/// assert_eq!(ink_to_substrate("Balance"), "T::Balance");
/// assert_eq!(ink_to_substrate("UnknownType"), "UnknownType");
/// ```
pub fn ink_to_substrate(type_str: &str) -> String {
    map_type(&initialize_type_mapper(), type_str)
}
