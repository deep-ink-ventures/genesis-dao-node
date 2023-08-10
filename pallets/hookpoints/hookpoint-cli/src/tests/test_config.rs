use serde_json::{json, to_string, to_string_pretty};
use std::fs;
use crate::config::models::{Definitions, PalletFunction};

#[test]
fn test_definitions_serialization() {
    let definitions = Definitions {
        name: "TestName".to_string(),
        pallets: std::collections::HashMap::new(),
    };

    let serialized = to_string(&definitions).unwrap();
    let expected = json!({
        "name": "TestName",
        "pallets": {}
    }).to_string();

    assert_eq!(serialized, expected);
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
fn test_definitions_write_to_file() {
    let definitions = Definitions {
        name: "TestName".to_string(),
        pallets: std::collections::HashMap::new(),
    };

    definitions.write_to_file();

    let content = fs::read_to_string("hookpoints.json").unwrap();
    let expected = to_string_pretty(&definitions).unwrap();

    assert_eq!(content, expected);
}

#[test]
fn test_definitions_add_pallet_function() {
    let mut definitions = Definitions {
        name: "TestName".to_string(),
        pallets: std::collections::HashMap::new(),
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
