use toml::map::Map;
use toml::Value;
use crate::config::models::Definitions;
use crate::utils::camel_case_to_kebap;


pub fn create_contracts_toml(definitions: &Definitions) -> String {
    let ink_deps = &definitions.ink_dependencies;

    let mut toml = Map::new();

    // [package]
    let mut package = Map::new();
    package.insert("name".to_string(), Value::String(camel_case_to_kebap(&definitions.name)));
    package.insert("version".to_string(), Value::String("0.1.0".to_string()));
    package.insert("edition".to_string(), Value::String("2021".to_string()));
    toml.insert("package".to_string(), Value::Table(package));

    // [dependencies]
    let mut dependencies = Map::new();
    dependencies.insert("ink".to_string(), Value::String(format!("{{ version = \"{}\", default-features = false }}", ink_deps.ink_version)));
    dependencies.insert("ink-primitives".to_string(), Value::String(format!("{{ version = \"{}\", default-features = false }}", ink_deps.ink_primitives_version)));
    dependencies.insert("scale".to_string(), Value::String(format!("{{ package = \"parity-scale-codec\", version = \"{}\", default-features = false, features = [\"derive\"] }}", ink_deps.scale_version)));
    dependencies.insert("scale-info".to_string(), Value::String(format!("{{ version = \"{}\", default-features = false, features = [\"derive\"], optional = true }}", ink_deps.scale_info_version)));
    toml.insert("dependencies".to_string(), Value::Table(dependencies));

    // [lib]
    let mut lib = Map::new();
    lib.insert("path".to_string(), Value::String("lib.rs".to_string()));
    toml.insert("lib".to_string(), Value::Table(lib));

    // [features]
    let mut features = Map::new();
    let std_features = vec![
        "ink/std".to_string(),
        "ink_primitives/std".to_string(),
        "scale/std".to_string(),
        "scale-info/std".to_string(),
    ];
    features.insert("default".to_string(), Value::Array(vec![Value::String("std".to_string())]));
    features.insert("std".to_string(), Value::Array(std_features.into_iter().map(Value::String).collect()));
    features.insert("ink-as-dependency".to_string(), Value::Array(vec![]));
    toml.insert("features".to_string(), Value::Table(features));

    toml::to_string(&Value::Table(toml)).unwrap()
}
