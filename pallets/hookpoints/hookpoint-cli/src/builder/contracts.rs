use toml::de::Error;
use crate::config::models::Definitions;
use crate::utils::camel_case_to_kebab;

pub fn create_contracts_toml(definitions: &Definitions) -> Result<String, toml::de::Error> {
    let ink_deps = &definitions.ink_dependencies;

    let toml_string = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
ink = {{ version = "{}", default-features = false }}
ink-primitives = {{ version = "{}", default-features = false }}
scale = {{ package = "parity-scale-codec", version = "{}", default-features = false, features = ["derive"] }}
scale-info = {{ version = "{}", default-features = false, features = ["derive"], optional = true }}

[lib]
path = "lib.rs"

[features]
default = ["std"]
std = [
    "ink/std",
    "ink_primitives/std",
    "scale/std",
    "scale-info/std",
]
ink-as-dependency = []
"#, camel_case_to_kebab(&definitions.name), ink_deps.ink_version, ink_deps.ink_primitives_version, ink_deps.scale_version, ink_deps.scale_info_version);

    // Now, let's validate the generated TOML
    let parsed: Result<toml::Value, Error> = toml::from_str(&toml_string);

    match parsed {
        Ok(_) => Ok(toml_string),
        Err(err) => Err(err)
    }
}
