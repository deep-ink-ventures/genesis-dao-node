use crate::builder::hooks::create_hooks;
use crate::builder::mapper::{ink_to_substrate, substrate_to_ink};
use crate::builder::contracts::create_contracts_toml;
use crate::config::models::{Definitions, FunctionArgument, InkDependencies, PalletFunction, ReturnValue};
use crate::utils::camel_case_to_kebab;

#[test]
fn test_ink_to_substrate() {
    // Known mappings
    assert_eq!(ink_to_substrate("Balance"), "T::Balance");
    assert_eq!(ink_to_substrate("AccountId"), "T::AccountId");

    // Unknown mapping, should return the same type string
    assert_eq!(ink_to_substrate("UnknownType"), "UnknownType");
}

#[test]
fn test_substrate_to_ink() {
    // Known reverse mappings
    assert_eq!(substrate_to_ink("T::Balance"), "Balance");
    assert_eq!(substrate_to_ink("T::AccountId"), "AccountId");

    // Unknown mapping, should return the same type string
    assert_eq!(substrate_to_ink("UnknownType"), "UnknownType");
}

#[test]
fn test_create_hooks() {
    let pallet_function = PalletFunction {
        hook_point: "test_hook_point".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "arg1".to_string(),
                type_: "Balance".to_string(),
            },
            FunctionArgument {
                name: "arg2".to_string(),
                type_: "AccountId".to_string(),
            },
        ],
        returns: Some(ReturnValue {
            default: "DefaultReturn".to_string(),
            type_: "Balance".to_string(),
        }),
    };
    let mut pallets = std::collections::HashMap::new();
    pallets.insert("TestPallet".to_string(), vec![pallet_function]);
    let config = Definitions {
        name: "TestConfig".to_string(),
        pallets,
        ink_dependencies: InkDependencies::default()
    };

    let hooks = create_hooks(config);
    assert_eq!(hooks.len(), 1);
    assert!(hooks.contains_key("TestPallet"));

    let content = &hooks["TestPallet"];

    // Verify the imports
    assert!(content.contains("use crate::Config;"));
    assert!(content.contains("use pallet_hookpoints::Pallet as HP;"));

    // Verify the function signature
    assert!(content.contains("pub fn test_hook_point<T: Config>(owner: T::AccountId, signer: T::AccountId, arg1: T::Balance, arg2: T::AccountId) -> T::Balance"));

    // Verify the function body for HP initialization
    assert!(content.contains("HP::<T>::create(\n\t\t\"TestConfig::test_hook_point\","));

    // Verify the function body for adding arguments
    assert!(content.contains(".add_arg::<T::Balance>(arg1)"));
    assert!(content.contains(".add_arg::<T::AccountId>(arg2);"));

    // Verify the function body for executing HP
    assert!(content.contains("HP::<T>::execute::<T::Balance>(hp).unwrap_or(DefaultReturn)"));
}

#[test]
fn test_create_hooks_no_returns_no_args() {
    let pallet_function = PalletFunction {
        hook_point: "test_hook_point_no_args".to_string(),
        arguments: vec![],
        returns: None,
    };
    let mut pallets = std::collections::HashMap::new();
    pallets.insert("TestPalletNoArgs".to_string(), vec![pallet_function]);
    let config = Definitions {
        name: "TestConfig".to_string(),
        pallets,
        ink_dependencies: InkDependencies::default()
    };

    let hooks = create_hooks(config);
    assert_eq!(hooks.len(), 1);
    assert!(hooks.contains_key("TestPalletNoArgs"));

    let content = &hooks["TestPalletNoArgs"];

    // Verify the imports
    assert!(content.contains("use crate::Config;"));
    assert!(content.contains("use pallet_hookpoints::Pallet as HP;"));

    // Verify the function signature (no arguments and no return type)
    assert!(content.contains("pub fn test_hook_point_no_args<T: Config>(owner: T::AccountId, signer: T::AccountId)"));

    // Verify the function body for HP initialization
    assert!(content.contains("HP::<T>::create(\n\t\t\"TestConfig::test_hook_point_no_args\","));

    // Verify the function body for executing HP (no return type)
    assert!(content.contains("HP::<T>::execute::<()>(hp)"));
}

#[test]
fn test_create_contracts_toml() {
    let definitions = Definitions {
        name: "TestProject".to_string(),
        pallets: std::collections::HashMap::new(),
        ink_dependencies: InkDependencies::default()
    };

    let toml_output = create_contracts_toml(&definitions).expect("to parse toml");

    // Now, check if the TOML contains the expected values
    assert!(toml_output.contains(&format!(r#"name = "{}""#, camel_case_to_kebab(&definitions.name))));
    assert!(toml_output.contains(&format!(r#"ink = {{ version = "{}", default-features = false }}"#, definitions.ink_dependencies.ink_version)));
    assert!(toml_output.contains(&format!(r#"ink-primitives = {{ version = "{}", default-features = false }}"#, definitions.ink_dependencies.ink_primitives_version)));
    assert!(toml_output.contains(&format!(r#"scale = {{ package = "parity-scale-codec", version = "{}", default-features = false, features = ["derive"] }}"#, definitions.ink_dependencies.scale_version)));
    assert!(toml_output.contains(&format!(r#"scale-info = {{ version = "{}", default-features = false, features = ["derive"], optional = true }}"#, definitions.ink_dependencies.scale_info_version)));
}
