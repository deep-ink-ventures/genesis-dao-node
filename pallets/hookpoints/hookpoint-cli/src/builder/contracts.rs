use toml::de::Error;
use crate::config::models::{Definitions, PalletFunction};
use crate::utils::{camel_case_to_kebab, camel_to_snake, get_default_for_ink_type, INK_PRIMITIVES, INK_TYPES};


fn generate_dependencies_toml(definitions: &Definitions, include_prelude: bool) -> String {
    let ink_deps = &definitions.ink_dependencies;

    // Check if we need to include ink_primitives
    let primitives_str = if definitions.contains_type(INK_PRIMITIVES) {
        format!("\nink_primitives = {{ version = \"{}\", default-features = false }}", ink_deps.ink_version)
    } else {
        String::new()
    };

    // Decide if we include ink_prelude
    let prelude_str = if include_prelude {
        format!("\nink_prelude = {{ version = \"{}\", default-features = false }}", ink_deps.ink_primitives_version)
    } else {
        String::new()
    };

    format!(
        r#"[dependencies]
ink = {{ version = "{}", default-features = false }}{}{}
scale = {{ package = "parity-scale-codec", version = "{}", default-features = false, features = ["derive"] }}
scale-info = {{ version = "{}", default-features = false, features = ["derive"], optional = true }}
"#, ink_deps.ink_version, prelude_str, primitives_str, ink_deps.scale_version, ink_deps.scale_info_version
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
authors = ["add your name here"]

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
"#, name_kebab, generate_dependencies_toml(&definitions, true), name_kebab, name_kebab, name_kebab, ink_deps.ink_version);

    // Validate the TOML
    let parsed: Result<toml::Value, Error> = toml::from_str(&toml_string);
    match parsed {
        Ok(_) => Ok(toml_string),
        Err(err) => Err(err)
    }
}

pub fn generate_contract_trait_toml(definitions: &Definitions) -> Result<String, toml::de::Error> {
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
"#, name_kebab, generate_dependencies_toml(&definitions, false));

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
                .map(|function| generate_trait_function_signature(function))
                .collect::<Vec<_>>()
        })
        .collect();

    // Check for types
    let mut ink_primitives: Vec<&str> = Vec::new();
    for prim in INK_PRIMITIVES.iter() {
        if definitions.contains_type(&[prim]) {
            ink_primitives.push(prim);
        }
    }

    let mut import_string = match ink_primitives.len() {
        0 => String::new(),
        1 => format!("use ink_primitives::{};\n", ink_primitives[0]),
        _ => format!("use ink_primitives::{{{}}};\n", ink_primitives.join(", ")),
    };

    if definitions.contains_type(&["Balance"]) {
        import_string.push_str("\ntype Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;\n");
    }

    format!(r##"#![cfg_attr(not(feature = "std"), no_std, no_main)]
{imports}
#[ink::trait_definition]
pub trait {trait_name} {{

{function_signatures}
}}"##,
            trait_name = definitions.name,
            function_signatures = function_signatures.join("\n\n"),
            imports = import_string
    )
}

fn generate_trait_function_signature(func: &PalletFunction) -> String {
    let args = func.arguments
        .iter()
        .map(|arg| format!("{name}: {type_}", name = arg.name, type_ = arg.type_))
        .collect::<Vec<_>>()
        .join(", ");

    let method_args = if args.is_empty() {
        "&self".to_string()
    } else {
        format!("&self, {}", args)
    };

    let return_type = if let Some(ret_val) = &func.returns {
        format!(" -> {}", ret_val.type_)
    } else {
        String::new()
    };

    format!(r##"    /// hook point for `{hook_point}` pallet
    #[ink(message)]
    fn {hook_point}({method_args}){return_type};"##,
            hook_point = func.hook_point,
            method_args = method_args,
            return_type = return_type
    )
}

fn generate_contract_functions(definitions: &Definitions)-> String {
    let functions: Vec<String> = definitions
        .pallets
        .iter()
        .flat_map(|(_, pallet_functions)| {
            pallet_functions
                .iter()
                .map(|function| generate_function_body(function))
                .collect::<Vec<_>>()
        })
        .collect();

    // Adjust the spaces for indentation to align with the `impl` block
    functions
        .iter()
        .map(|f| format!("        {}", f))
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn generate_function_body(func: &PalletFunction) -> String {
    let args = func
        .arguments
        .iter()
            .map(|arg| {
                if func.returns.is_some() && func.returns.as_ref().unwrap().default == arg.name {
                    format!("{name}: {type_}", name = arg.name, type_ = arg.type_)
                } else {
                    format!("_{name}: {type_}", name = arg.name, type_ = arg.type_)
                }
            })
        .collect::<Vec<_>>()
        .join(", ");

    let return_type = if let Some(ret_val) = &func.returns {
        if INK_TYPES.contains(&ret_val.default.as_str()) {
            format!(" -> {}", ret_val.type_)
        } else if func.arguments.iter().any(|arg| arg.name == ret_val.default) {
            format!(" -> {}", ret_val.type_)
        } else {
            format!(" -> {}", ret_val.type_)
        }
    } else {
        String::new()
    };

    let function_body = if let Some(ret_val) = &func.returns {
        if INK_TYPES.contains(&ret_val.default.as_str()) {
            get_default_for_ink_type(&ret_val.type_)
        } else {
            ret_val.default.clone()
        }
    } else {
        "// do nothing".to_string()
    };

    format!(
        r##"/// hook point for `{hook_point}` pallet
        #[ink(message)]
        fn {hook_point}(&self{args}){return_type} {{
            {function_body}
        }}"##,
        hook_point = func.hook_point,
        args = if args.is_empty() { String::new() } else { format!(", {}", args) },
        return_type = return_type,
        function_body = function_body
    )
}

fn generate_ink_test_functions(definitions: &Definitions) -> String {
    let tests: Vec<String> = definitions
        .pallets
        .iter()
        .flat_map(|(_, pallet_functions)| {
            pallet_functions
                .iter()
                .map(|function| generate_test_function(function, &definitions.name))
                .collect::<Vec<_>>()
        })
        .collect();

    tests.join("\n")
}

fn generate_test_function(func: &PalletFunction, contract_name: &str) -> String {
    let contract_instance = format!(
        "let {} = {}::new();",
        camel_to_snake(contract_name),
        contract_name
    );

    let arguments: Vec<String> = func
        .arguments
        .iter()
        .map(|arg| get_default_for_ink_type(&arg.type_))
        .collect();

    let expected_return = if let Some(ret_val) = &func.returns {
        get_default_for_ink_type(&ret_val.type_)
    } else {
        "()".to_string()
    };

    format!(
        r##"
        #[ink::test]
        fn test_{hook_point}_hookpoint() {{
            {contract_instance}
            assert_eq!({contract_snake_name}.{hook_point}({arguments}), {expected_return});
        }}"##,
        hook_point = func.hook_point,
        contract_instance = contract_instance,
        contract_snake_name = camel_to_snake(contract_name),
        arguments = arguments.join(", "),
        expected_return = expected_return
    )
}

pub fn generate_ink_boilerplate_contract(definitions: &Definitions) -> String {
    let functions = generate_contract_functions(definitions);
    let contract_name = &definitions.name;
    let contract_name_lower = camel_to_snake(contract_name);

    format!(
        r##"#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod {contract_name_lower} {{
    #[ink(storage)]
    pub struct {contract_name} {{}}

    impl {contract_name} {{
        #[ink(constructor)]
        pub fn new() -> Self {{
            Self {{}}
        }}
    }}

    impl {contract_name_lower}_contract_trait::{contract_name} for {contract_name} {{
{functions}
    }}

    #[cfg(test)]
    mod tests {{
        use super::*;
        use {contract_name_lower}_contract_trait::{contract_name} as Trait;
        {ink_test_functions}
    }}
}}"##,
        contract_name = contract_name,
        contract_name_lower = contract_name_lower,
        functions = functions,
        ink_test_functions = generate_ink_test_functions(&definitions)
    )
}
