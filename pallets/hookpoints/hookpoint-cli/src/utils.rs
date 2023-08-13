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

//! Utility functions for assisting with conversions, string manipulations,
//! and default value generations, specifically tailored for ink! types and naming conventions.

/// Constant array of ink! primitives.
pub const INK_PRIMITIVES: &[&str; 2] = &["AccountId", "Hash"];

/// Constant array of supported ink! types.
pub const INK_TYPES: [&str; 24] = [
    "bool",
    "u8", "u16", "u32", "u64", "u128",
    "i8", "i16", "i32", "i64", "i128",
    "Vec<u8>", "Vec<u16>", "Vec<u32>", "Vec<u64>", "Vec<u128>",
    "Vec<i8>", "Vec<i16>", "Vec<i32>", "Vec<i64>", "Vec<i128>",
    "AccountId",
    "Hash",
    "Balance",
];

/// Retrieves a default value for a given ink! type.
///
/// # Examples
///
/// ```
/// let default_value = get_default_for_ink_type("u32");
/// assert_eq!(default_value, "0");
/// ```
///
/// # Panics
///
/// This function will panic if provided with an unknown ink! type.
pub fn get_default_for_ink_type(type_str: &str) -> String {
    match type_str {
        "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" | "Balance" => "0".to_string(),
        "Vec<u8>" | "Vec<u32>" | "Vec<u64>" | "Vec<u128>" | "Vec<i8>" | "Vec<i16>" | "Vec<i32>" | "Vec<i64>" | "Vec<i128>" => "vec![]".to_string(),
        "AccountId" => "AccountId::from([0x01; 32])".to_string(),
        "Hash" => "Hash::default()".to_string(),
        _ => panic!("Unknown INK type: {}", type_str),
    }
}

/// Converts a CamelCase string to kebab-case.
///
/// # Examples
///
/// ```
/// let kebab_case_string = camel_case_to_kebab("CamelCaseString");
/// assert_eq!(kebab_case_string, "camel-case-string");
/// ```
pub fn camel_case_to_kebab(s: &str) -> String {
    let mut kebap = String::new();
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        if chars[i].is_uppercase() {
            // Add a '-' if it's not the first character and previous character is not uppercase
            if i != 0 && !chars[i - 1].is_uppercase() {
                kebap.push('-');
            }
            kebap.push(chars[i].to_ascii_lowercase());
        } else {
            kebap.push(chars[i]);
        }
    }

    kebap
}

/// Converts a CamelCase string to snake_case.
///
/// # Examples
///
/// ```
/// let snake_case_string = camel_to_snake("CamelCaseString");
/// assert_eq!(snake_case_string, "camel_case_string");
/// ```
pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_uppercase() {
            if !result.is_empty() {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}
