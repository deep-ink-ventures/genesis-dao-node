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

use std::collections::HashMap;
use crate::builder::hooks::create_hooks;
use crate::builder::mapper::{ink_to_substrate};
use crate::builder::contracts::{generate_contract_boilerplate_toml, generate_contract_trait_toml, generate_ink_boilerplate_contract, generate_ink_trait};
use crate::config::models::{Definitions, FunctionArgument, InkDependencies, PalletFunction, ReturnValue};

#[test]
fn test_ink_to_substrate() {
    // Known mappings
    assert_eq!(ink_to_substrate("Balance"), "T::Balance");
    assert_eq!(ink_to_substrate("AccountId"), "T::AccountId");

    // Unknown mapping, should return the same type string
    assert_eq!(ink_to_substrate("UnknownType"), "UnknownType");
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
        ink_dependencies: InkDependencies::default(),
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
        ink_dependencies: InkDependencies::default(),
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
fn test_generate_ink_trait() {
    let mut pallets: HashMap<String, Vec<PalletFunction>> = HashMap::new();

    // Sample pallet functions
    let func_no_args_no_return = PalletFunction {
        hook_point: "func_a".to_string(),
        arguments: vec![],
        returns: None,
    };

    let func_only_args = PalletFunction {
        hook_point: "func_b".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "arg1".to_string(),
                type_: "u32".to_string(),
            }
        ],
        returns: None,
    };

    let func_only_return = PalletFunction {
        hook_point: "func_c".to_string(),
        arguments: vec![],
        returns: Some(ReturnValue {
            default: "0".to_string(),
            type_: "u32".to_string(),
        }),
    };

    let func_args_and_return = PalletFunction {
        hook_point: "func_d".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "account".to_string(),
                type_: "AccountId".to_string(),
            }
        ],
        returns: Some(ReturnValue {
            default: "0".to_string(),
            type_: "Balance".to_string(),
        }),
    };

    pallets.insert("sample_pallet".to_string(), vec![
        func_no_args_no_return,
        func_only_args,
        func_only_return,
        func_args_and_return,
    ]);

    let definitions = Definitions {
        name: "SampleContract".to_string(),
        ink_dependencies: InkDependencies::default(),
        pallets,
    };

    let trait_def = generate_ink_trait(&definitions);
    let expected = r##"#![cfg_attr(not(feature = "std"), no_std, no_main)]
use ink_primitives::AccountId;

type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

#[ink::trait_definition]
pub trait SampleContract {

    /// hook point for `func_a` pallet
    #[ink(message)]
    fn func_a(&self);

    /// hook point for `func_b` pallet
    #[ink(message)]
    fn func_b(&self, arg1: u32);

    /// hook point for `func_c` pallet
    #[ink(message)]
    fn func_c(&self) -> u32;

    /// hook point for `func_d` pallet
    #[ink(message)]
    fn func_d(&self, account: AccountId) -> Balance;
}"##;

    assert_eq!(trait_def, expected);
}

#[test]
fn test_generate_ink_trait_ink_primitives_inclusion() {
    let func_no_special_type = PalletFunction {
        hook_point: "func_a".to_string(),
        arguments: vec![],
        returns: None,
    };

    let func_with_account_id = PalletFunction {
        hook_point: "func_b".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "account".to_string(),
                type_: "AccountId".to_string(),
            }
        ],
        returns: None,
    };

    let func_with_hash = PalletFunction {
        hook_point: "func_c".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "hash_val".to_string(),
                type_: "Hash".to_string(),
            }
        ],
        returns: None,
    };

    let func_with_balance = PalletFunction {
        hook_point: "func_d".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "balance_val".to_string(),
                type_: "Balance".to_string(),
            }
        ],
        returns: None,
    };

    // Scenario 1: No ink primtives types
    let mut pallets = HashMap::new();
    pallets.insert("sample_pallet".to_string(), vec![func_no_special_type.clone()]);
    let definitions = Definitions {
        name: "SampleContract".to_string(),
        ink_dependencies: InkDependencies::default(),
        pallets: pallets.clone(),
    };
    assert!(!generate_ink_trait(&definitions).contains("ink_primitives"));
    // lets test that balance hasn't been injected here:
    assert!(!generate_ink_trait(&definitions).contains("type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;"));

    // Scenario 2: Only AccountId
    pallets.insert("sample_pallet".to_string(), vec![func_with_account_id.clone()]);
    let definitions = Definitions {
        name: "SampleContract".to_string(),
        ink_dependencies: InkDependencies::default(),
        pallets: pallets.clone(),
    };
    assert!(generate_ink_trait(&definitions).contains("use ink_primitives::AccountId;"));

    // Scenario 3: Both AccountId and Hash
    pallets.insert("sample_pallet".to_string(), vec![func_with_account_id.clone(), func_with_hash.clone()]);
    let definitions = Definitions {
        name: "SampleContract".to_string(),
        ink_dependencies: InkDependencies::default(),
        pallets: pallets.clone(),
    };
    assert!(generate_ink_trait(&definitions).contains("use ink_primitives::{AccountId, Hash};"));

    // Scenario 4: Both AccountId and Hash
    pallets.insert("sample_pallet".to_string(), vec![func_with_balance.clone()]);
    let definitions = Definitions {
        name: "SampleContract".to_string(),
        ink_dependencies: InkDependencies::default(),
        pallets,
    };
    assert!(generate_ink_trait(&definitions).contains("type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;"));

}


#[test]
fn test_generate_contract_trait_toml() {
    let definitions = Definitions {
        name: "GenesisDao".to_string(),
        pallets: HashMap::new(),
        ink_dependencies: InkDependencies::default(),
    };

    let output = generate_contract_trait_toml(&definitions).unwrap();
    let ink_deps = InkDependencies::default();
    let expected_output = format!(r#"[package]
name = "genesis-dao-contract-trait"
version = "0.1.0"
edition = "2021"

[dependencies]
ink = {{ version = "{}", default-features = false }}
scale = {{ package = "parity-scale-codec", version = "{}", default-features = false, features = ["derive"] }}
scale-info = {{ version = "{}", default-features = false, features = ["derive"], optional = true }}

[lib]
path = "lib.rs"

[features]
default = ["std"]
std = [
    "ink/std",
    "scale/std",
    "scale-info/std",
]
ink-as-dependency = []

[workspace]
"#, ink_deps.ink_version, ink_deps.scale_version, ink_deps.scale_info_version);

    assert_eq!(output, expected_output);
}

#[test]
fn test_generate_contract_trait_toml_with_ink_primitives() {
    // Create a pallet function that uses the AccountId type
    let pallet_function = PalletFunction {
        hook_point: "hook_point".to_string(),
        arguments: vec![FunctionArgument {
            name: "arg1".to_string(),
            type_: "AccountId".to_string(),
        }],
        returns: None,
    };

    let mut pallets = std::collections::HashMap::new();
    pallets.insert("TestPallet".to_string(), vec![pallet_function]);

    let definitions = Definitions {
        name: "GenesisDao".to_string(),
        pallets,
        ink_dependencies: InkDependencies::default(),
    };

    let output = generate_contract_trait_toml(&definitions).unwrap();
    let ink_deps = InkDependencies::default();
    let expected_output = format!(r#"[package]
name = "genesis-dao-contract-trait"
version = "0.1.0"
edition = "2021"

[dependencies]
ink = {{ version = "{}", default-features = false }}
ink_primitives = {{ version = "{}", default-features = false }}
scale = {{ package = "parity-scale-codec", version = "{}", default-features = false, features = ["derive"] }}
scale-info = {{ version = "{}", default-features = false, features = ["derive"], optional = true }}

[lib]
path = "lib.rs"

[features]
default = ["std"]
std = [
    "ink/std",
    "scale/std",
    "scale-info/std",
]
ink-as-dependency = []

[workspace]
"#, ink_deps.ink_version, ink_deps.ink_version, ink_deps.scale_version, ink_deps.scale_info_version);

    assert_eq!(output, expected_output);
}


#[test]
fn test_generate_contract_boilerplate_toml() {
    let definitions = Definitions {
        name: "GenesisDao".to_string(),
        pallets: std::collections::HashMap::new(),
        ink_dependencies: InkDependencies::default(),
    };

    let output = generate_contract_boilerplate_toml(&definitions);
    let ink_deps = &definitions.ink_dependencies;

    let expected_output = format!(r#"[package]
name = "genesis-dao-contract"
version = "0.1.0"
edition = "2021"
authors = ["add your name here"]

[dependencies]
ink = {{ version = "{}", default-features = false }}
ink_prelude = {{ version = "{}", default-features = false }}
scale = {{ package = "parity-scale-codec", version = "{}", default-features = false, features = ["derive"] }}
scale-info = {{ version = "{}", default-features = false, features = ["derive"], optional = true }}

genesis-dao-contract-trait = {{ package = "genesis-dao-contract-trait", default-features = false, path = "../genesis-dao-contract-trait" }}

[dev-dependencies]
ink_e2e = "{}"

[lib]
path = "lib.rs"

[features]
default = ["std"]
std = [
    "ink/std",
    "ink_prelude/std",
    "scale/std",
    "scale-info/std",
]
ink-as-dependency = []
e2e-tests = []

[workspace]
"#, ink_deps.ink_version, ink_deps.ink_primitives_version, ink_deps.scale_version, ink_deps.scale_info_version, ink_deps.ink_version);


    assert_eq!(output.unwrap(), expected_output);
}

#[test]
fn test_generate_ink_boilerplate_contract() {
    let mut pallets: HashMap<String, Vec<PalletFunction>> = HashMap::new();

    // Defining pallet functions based on the given scenarios
    let func_no_args_no_return = PalletFunction {
        hook_point: "func_a".to_string(),
        arguments: vec![],
        returns: None,
    };

    let func_account_id_return = PalletFunction {
        hook_point: "func_b".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "account".to_string(),
                type_: "AccountId".to_string(),
            },
            FunctionArgument {
                name: "hash_val".to_string(),
                type_: "Hash".to_string(),
            },
            FunctionArgument {
                name: "value".to_string(),
                type_: "u128".to_string(),
            },
        ],
        returns: Some(ReturnValue {
            default: "account".to_string(),
            type_: "AccountId".to_string(),
        }),
    };

    let func_vec_u8_return = PalletFunction {
        hook_point: "func_c".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "arg1".to_string(),
                type_: "u32".to_string(),
            },
            FunctionArgument {
                name: "arg2".to_string(),
                type_: "Hash".to_string(),
            },
        ],
        returns: Some(ReturnValue {
            default: "Vec<u8>".to_string(),
            type_: "Vec<u8>".to_string(),
        }),
    };

    let func_u32_return = PalletFunction {
        hook_point: "func_d".to_string(),
        arguments: vec![],
        returns: Some(ReturnValue {
            default: "u32".to_string(),
            type_: "u32".to_string(),
        }),
    };

    let func_no_return = PalletFunction {
        hook_point: "func_e".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "arg1".to_string(),
                type_: "u64".to_string(),
            },
            FunctionArgument {
                name: "arg2".to_string(),
                type_: "Hash".to_string(),
            },
        ],
        returns: None,
    };

    let func_account_id_default_return = PalletFunction {
        hook_point: "func_e".to_string(),
        arguments: vec![
            FunctionArgument {
                name: "account".to_string(),
                type_: "AccountId".to_string(),
            },
            FunctionArgument {
                name: "hash_val".to_string(),
                type_: "Hash".to_string(),
            },
            FunctionArgument {
                name: "value".to_string(),
                type_: "u128".to_string(),
            },
        ],
        returns: Some(ReturnValue {
            default: "AccountId".to_string(),
            type_: "AccountId".to_string(),
        }),
    };

    pallets.insert("sample_pallet".to_string(), vec![
        func_no_args_no_return,
        func_account_id_return,
        func_vec_u8_return,
        func_u32_return,
        func_no_return,
        func_account_id_default_return
    ]);

    let definitions = Definitions {
        name: "SampleContract".to_string(),
        ink_dependencies: InkDependencies::default(),
        pallets,
    };

    let boilerplate_contract = generate_ink_boilerplate_contract(&definitions);

    // Check for the correct module name
    assert!(boilerplate_contract.contains("mod sample_contract {"));

    // Check for the correct struct declaration
    assert!(boilerplate_contract.contains("pub struct SampleContract {}"));

    // Check for the correct trait implementation
    assert!(boilerplate_contract.contains("impl sample_contract_contract_trait::SampleContract for SampleContract {"));


    // Assertions to ensure the presence of the entire function bodies
    assert!(boilerplate_contract.contains(r"fn func_a(&self) {
            // do nothing
        }"));

    assert!(boilerplate_contract.contains(r" fn func_b(&self, account: AccountId, _hash_val: Hash, _value: u128) -> AccountId {
            account
        }"));

    assert!(boilerplate_contract.contains(r"fn func_c(&self, _arg1: u32, _arg2: Hash) -> Vec<u8> {
            vec![]
        }"));

    assert!(boilerplate_contract.contains(r"fn func_d(&self) -> u32 {
            0
        }"));

    assert!(boilerplate_contract.contains(r"fn func_e(&self, _account: AccountId, _hash_val: Hash, _value: u128) -> AccountId {
            AccountId::from([0x01; 32])
        }"));

      // Assertions to ensure the presence of the test functions
    assert!(boilerplate_contract.contains(r"#[ink::test]
        fn test_func_a_hookpoint() {
            let sample_contract = SampleContract::new();
            assert_eq!(sample_contract.func_a(), ());
        }"));

    assert!(boilerplate_contract.contains(r"#[ink::test]
        fn test_func_b_hookpoint() {
            let sample_contract = SampleContract::new();
            assert_eq!(sample_contract.func_b(AccountId::from([0x01; 32]), Hash::default(), 0), AccountId::from([0x01; 32]));
        }"));

    assert!(boilerplate_contract.contains(r"#[ink::test]
        fn test_func_c_hookpoint() {
            let sample_contract = SampleContract::new();
            assert_eq!(sample_contract.func_c(0, Hash::default()), vec![]);
        }"));

    assert!(boilerplate_contract.contains(r"#[ink::test]
        fn test_func_d_hookpoint() {
            let sample_contract = SampleContract::new();
            assert_eq!(sample_contract.func_d(), 0);
        }"));

    assert!(boilerplate_contract.contains(r"#[ink::test]
        fn test_func_e_hookpoint() {
            let sample_contract = SampleContract::new();
            assert_eq!(sample_contract.func_e(0, Hash::default()), ());
        }"));

    assert!(boilerplate_contract.contains(r"#[ink::test]
        fn test_func_e_hookpoint() {
            let sample_contract = SampleContract::new();
            assert_eq!(sample_contract.func_e(AccountId::from([0x01; 32]), Hash::default(), 0), AccountId::from([0x01; 32]));
        }"));
}
