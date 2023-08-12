use std::collections::HashMap;
use serde_json::json;
use std::fs::File;
use std::io::Read;
use tempfile::tempdir;
use std::path::PathBuf;
use crate::config::models::{Definitions, FunctionArgument, InkDependencies, PalletFunction, ReturnValue};

#[test]
fn test_definitions_serialization() {
    let definitions = Definitions {
        name: "TestName".to_string(),
        pallets: std::collections::HashMap::new(),
        ink_dependencies: InkDependencies::default(),
    };

    let serialized = serde_json::to_string(&definitions).unwrap();

    let ink_dependencies = InkDependencies::default();
    let expected = json!({
        "name": "TestName",
        "ink_dependencies": {
            "ink_version": ink_dependencies.ink_version,
            "ink_primitives_version": ink_dependencies.ink_primitives_version,
            "scale_version": ink_dependencies.scale_version,
            "scale_info_version": ink_dependencies.scale_info_version,
        },
        "pallets": {}
    }).to_string();

    // Deserialize both strings back to Definitions struct
    let actual_data: Definitions = serde_json::from_str(&serialized).unwrap();
    let expected_data: Definitions = serde_json::from_str(&expected).unwrap();

    // Compare the deserialized data
    assert_eq!(actual_data, expected_data);
}

// Definitions Tests
#[test]
fn test_definitions_new() {
    let name = "TestName".to_string();
    let pallets = std::collections::HashMap::new();
    let definitions = Definitions::new(name.clone(), pallets.clone());

    assert_eq!(definitions.name, name);
    assert_eq!(definitions.pallets, pallets);
}


#[test]
fn test_write_to_file_in_specified_directory() {
    // Setup a temporary directory
    let dir = tempdir().unwrap();

    let definitions = Definitions {
        name: "TestName".to_string(),
        pallets: std::collections::HashMap::new(),
        ink_dependencies: InkDependencies::default(),
    };

    definitions.write_to_file(&Some(dir.path()));

    let file_path = dir.path().join("hookpoints.json");
    let mut file = File::open(file_path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    assert_eq!(content, serde_json::to_string_pretty(&definitions).unwrap());
}

#[test]
fn test_write_to_file_in_default_directory() {
    // Setup a temporary directory
    let dir = tempdir().unwrap();

    // Change current directory to the temporary directory
    std::env::set_current_dir(&dir).unwrap();

    let definitions = Definitions {
        name: "TestName".to_string(),
        pallets: std::collections::HashMap::new(),
        ink_dependencies: InkDependencies::default(),
    };

    definitions.write_to_file::<PathBuf>(&None); // Using a temporary directory as the current directory

    let file_path = dir.path().join("hookpoints.json");
    let mut file = File::open(file_path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    assert_eq!(content, serde_json::to_string_pretty(&definitions).unwrap());
}


#[test]
fn test_definitions_add_pallet_function() {
    let mut definitions = Definitions {
        name: "TestName".to_string(),
        pallets: std::collections::HashMap::new(),
        ink_dependencies: InkDependencies::default(),
    };

    let pallet_function = PalletFunction {
        hook_point: "HookPoint1".to_string(),
        arguments: vec![],
        returns: None,
    };

    definitions.add_pallet_function("Pallet1".to_string(), pallet_function.clone());

    assert_eq!(definitions.pallets["Pallet1"], vec![pallet_function.clone()]);

    // Test adding another function to the same pallet
    let pallet_function2 = PalletFunction {
        hook_point: "HookPoint2".to_string(),
        arguments: vec![],
        returns: None,
    };
    definitions.add_pallet_function("Pallet1".to_string(), pallet_function2.clone());
    assert_eq!(definitions.pallets["Pallet1"], vec![pallet_function, pallet_function2]);
}

fn mock_definitions() -> Definitions {
    let mut pallets = HashMap::new();
    pallets.insert(
        "test_pallet".to_string(),
        vec![
            PalletFunction {
                hook_point: "hook1".to_string(),
                arguments: vec![
                    FunctionArgument { name: "arg1".to_string(), type_: "AccountId".to_string() },
                    FunctionArgument { name: "arg2".to_string(), type_: "u32".to_string() },
                ],
                returns: Some(ReturnValue { default: "default_value".to_string(), type_: "Hash".to_string() }),
            },
            PalletFunction {
                hook_point: "hook2".to_string(),
                arguments: vec![
                    FunctionArgument { name: "arg3".to_string(), type_: "String".to_string() },
                ],
                returns: None,
            },
        ],
    );

    Definitions {
        name: "test".to_string(),
        ink_dependencies: InkDependencies::default(),
        pallets,
    }
}

#[test]
fn test_extract_types() {
    let definitions = mock_definitions();
    let types = definitions.extract_types();

    assert_eq!(types, vec!["AccountId", "u32", "Hash", "String"]);
}

#[test]
fn test_contains_type() {
    let definitions = mock_definitions();

    assert!(definitions.contains_type(&["AccountId", "Hash"]));
    assert!(definitions.contains_type(&["String"]));
    assert!(!definitions.contains_type(&["Vec<u8>"]));
}
