#[cfg(test)]
use std::fs;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use crate::config::models::InkDependencies;
use crate::environment::{get_pallets, load_definitions};
use crate::interactive::{CAMEL_CASE_REGEX, SNAKE_CASE_REGEX};
use crate::utils::{camel_case_to_kebab, camel_to_snake, get_default_for_ink_type};

#[test]
fn test_camel_case_to_kebap() {
    assert_eq!(camel_case_to_kebab("CamelCaseString"), "camel-case-string");
    assert_eq!(camel_case_to_kebab("Simple"), "simple");
    assert_eq!(camel_case_to_kebab("MultipleUppercaseLETTERS"), "multiple-uppercase-letters");
    assert_eq!(camel_case_to_kebab("WithNumbers123"), "with-numbers123");
    assert_eq!(camel_case_to_kebab("Already-kebap"), "already-kebap");
    assert_eq!(camel_case_to_kebab(""), "");
}

#[test]
    fn test_camel_to_snake() {
    assert_eq!(camel_to_snake("SampleContract"), "sample_contract");
    assert_eq!(camel_to_snake("CamelCaseHere"), "camel_case_here");
    assert_eq!(camel_to_snake("AlreadySnake"), "already_snake");
    assert_eq!(camel_to_snake("NoChange"), "no_change");
    assert_eq!(camel_to_snake("A"), "a");
    assert_eq!(camel_to_snake(""), "");
}

#[test]
fn test_get_pallets() {
    // Create a temporary directory.
    let dir = tempdir().unwrap();

    // Create a "pallets" directory inside the temporary directory.
    let pallets_path = dir.path().join("pallets");
    fs::create_dir(&pallets_path).unwrap();

    // Create some directories and files inside the "pallets" directory.
    let pallet1 = pallets_path.join("pallet1");
    fs::create_dir(&pallet1).unwrap();

    let pallet2 = pallets_path.join("pallet2");
    fs::create_dir(&pallet2).unwrap();

    let file1 = pallets_path.join("file1.txt");
    fs::write(&file1, b"Some content").unwrap();

    // Call the function and check its output.
    let result = get_pallets(&Some(dir.path())).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result.contains_key("pallet1"));
    assert!(result.contains_key("pallet2"));
    assert!(!result.contains_key("file1.txt"));
}

#[test]
fn test_load_definitions_success() {
    // Setup a temporary directory with a hookpoints.json file
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("hookpoints.json");
    let mut file = File::create(&file_path).unwrap();

    let default_ink_dependencies = InkDependencies::default();
    let serialized_data = format!(
        r#"{{
            "name": "TestName",
            "ink_dependencies": {{
                "ink_version": "{}",
                "ink_primitives_version": "{}",
                "scale_version": "{}",
                "scale_info_version": "{}"
            }},
            "pallets": {{}}
        }}"#,
        default_ink_dependencies.ink_version,
        default_ink_dependencies.ink_primitives_version,
        default_ink_dependencies.scale_version,
        default_ink_dependencies.scale_info_version
    );

    file.write_all(serialized_data.as_bytes()).unwrap();

    let loaded_definitions = load_definitions(&Some(dir.path())).unwrap();
    assert_eq!(loaded_definitions.name, "TestName");
    assert_eq!(loaded_definitions.pallets.len(), 0);
    assert_eq!(loaded_definitions.ink_dependencies, InkDependencies::default());
}

#[test]
fn test_load_definitions_missing_file() {
    let dir = tempdir().unwrap();
    let result = load_definitions(&Some(dir.path()));
    assert!(result.is_err());
}

#[test]
fn test_load_definitions_invalid_content() {
    // Setup a temporary directory with an invalid hookpoints.json file
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("hookpoints.json");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, r#"Invalid JSON Content"#).unwrap();

    let result = load_definitions(&Some(dir.path()));
    assert!(result.is_err());
}

#[test]
fn test_camel_case_regex() {
    let camel_case_regex = regex::Regex::new(CAMEL_CASE_REGEX).unwrap();

    assert!(camel_case_regex.is_match("CamelCaseString"));
    assert!(camel_case_regex.is_match("Simple"));
    assert!(camel_case_regex.is_match("Camel"));
    assert!(!camel_case_regex.is_match("camelCaseString"));
    assert!(!camel_case_regex.is_match("snake_case_string"));
    assert!(!camel_case_regex.is_match("WithNumbers123"));
}

#[test]
fn test_snake_case_regex() {
    let snake_case_regex = regex::Regex::new(SNAKE_CASE_REGEX).unwrap();

    assert!(snake_case_regex.is_match("snake_case_string"));
    assert!(snake_case_regex.is_match("simple"));
    assert!(!snake_case_regex.is_match("CamelCaseString"));
    assert!(!snake_case_regex.is_match("snakeCaseString"));
    assert!(!snake_case_regex.is_match("With_Numbers123"));
}

#[test]
fn test_default_for_ink_type() {
    assert_eq!(get_default_for_ink_type("u8"), "0");
    assert_eq!(get_default_for_ink_type("u128"), "0");
    assert_eq!(get_default_for_ink_type("i64"), "0");
    assert_eq!(get_default_for_ink_type("Balance"), "0");
    assert_eq!(get_default_for_ink_type("Vec<u8>"), "vec![]");
    assert_eq!(get_default_for_ink_type("Vec<i128>"), "vec![]");
    assert_eq!(get_default_for_ink_type("AccountId"), "AccountId::from([0x01; 32])");
    assert_eq!(get_default_for_ink_type("Hash"), "Hash::default()");
}

#[test]
#[should_panic(expected = "Unknown INK type: u256")]
fn test_default_for_unknown_type() {
    get_default_for_ink_type("u256");
}