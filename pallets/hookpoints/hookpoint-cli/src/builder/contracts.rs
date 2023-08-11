use toml::de::Error;
use crate::config::models::{Definitions, InkDependencies, PalletFunction};
use crate::utils::camel_case_to_kebab;

fn generate_dependencies_toml(ink_deps: &InkDependencies, include_prelude: bool) -> String {
    let prelude_str = if include_prelude {
        format!("\nink_prelude = {{ version = \"{}\", default-features = false }}", ink_deps.ink_primitives_version)
    } else {
        String::new()
    };

    format!(
        r#"[dependencies]
ink = {{ version = "{}", default-features = false }}{}
scale = {{ package = "parity-scale-codec", version = "{}", default-features = false, features = ["derive"] }}
scale-info = {{ version = "{}", default-features = false, features = ["derive"], optional = true }}
"#, ink_deps.ink_version, prelude_str, ink_deps.scale_version, ink_deps.scale_info_version
    )
}


pub fn generate_contract_boilerplate_toml(definitions: &Definitions) -> Result<String, toml::de::Error> {
    let ink_deps = &definitions.ink_dependencies;
    let name_kebab = camel_case_to_kebab(&definitions.name);

    let toml_string = format!(
        r#"[package]
name = "{}-contract-boilerplate"
version = "0.1.0"
edition = "2021"

{}
{}-contract-trait = {{ package = "{}-contract-trait", default-features = false, path = "../{}-contract-trait" }}

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
"#, name_kebab, generate_dependencies_toml(ink_deps, true), name_kebab, name_kebab, name_kebab, ink_deps.ink_version);

    // Validate the TOML
    let parsed: Result<toml::Value, Error> = toml::from_str(&toml_string);
    match parsed {
        Ok(_) => Ok(toml_string),
        Err(err) => Err(err)
    }
}

pub fn generate_contract_trait_toml(definitions: &Definitions) -> Result<String, toml::de::Error> {
    let ink_deps = &definitions.ink_dependencies;
    let name_kebab = camel_case_to_kebab(&definitions.name);

    let toml_string = format!(
        r#"[package]
name = "{}-contract-trait"
version = "0.1.0"
edition = "2021"

{}
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
"#, name_kebab, generate_dependencies_toml(ink_deps, false));

    // Validate the TOML
    let parsed: Result<toml::Value, Error> = toml::from_str(&toml_string);
    match parsed {
        Ok(_) => Ok(toml_string),
        Err(err) => Err(err)
    }
}

pub fn generate_ink_trait(definitions: &Definitions) -> String {
    let function_signatures: Vec<String> = definitions.pallets
        .iter()
        .flat_map(|(_, pallet_functions)| {
            pallet_functions
                .iter()
                .map(|function| generate_function_signature(function))
                .collect::<Vec<_>>()
        })
        .collect();

    format!(r##"#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink_primitives::{{AccountId}};

#[ink::trait_definition]
pub trait {trait_name} {{

{function_signatures}
}}"##,
            trait_name = definitions.name,
            function_signatures = function_signatures.join("\n\n")
    )
}

fn generate_function_signature(func: &PalletFunction) -> String {
    let args = func.arguments
        .iter()
        .map(|arg| format!("{name}: {type_}", name = arg.name, type_ = arg.type_))
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = if let Some(ret_val) = &func.returns {
        format!(" -> {}", ret_val.type_)
    } else {
        String::new()
    };

    format!(r##"    /// hook point for `{hook_point}` pallet
    #[ink(message)]
    fn {hook_point}({args}){return_type};"##,
            hook_point = func.hook_point,
            args = args,
            return_type = return_type
    )
}
