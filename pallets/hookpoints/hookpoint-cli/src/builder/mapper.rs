use std::collections::HashMap;

fn initialize_type_mapper() -> HashMap<String, String> {
    let mut mapper = HashMap::new();

    // Specialized types for substrate/ink
    mapper.insert("Balance".to_string(), "T::Balance".to_string());
    mapper.insert("AccountId".to_string(), "T::AccountId".to_string());

    mapper
}

fn map_type(mapper: &HashMap<String, String>, type_str: &str) -> String {
    mapper.get(type_str).cloned().unwrap_or_else(|| type_str.to_string())
}

pub fn ink_to_substrate(type_str: &str) -> String {
    map_type(&initialize_type_mapper(), type_str)
}
