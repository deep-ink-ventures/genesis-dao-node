use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::config::models::Definitions;

pub(crate) fn get_pallets<P: AsRef<Path>>(substrate_dir: &Option<P>) -> std::io::Result<HashMap<String, PathBuf>> {
    let dir = match substrate_dir {
        None => std::env::current_dir()?,
        Some(ref substrate_dir) => PathBuf::from(substrate_dir.as_ref()),
    };
    let mut dirs: HashMap<String, PathBuf> = HashMap::new();
    let pallets_dir = dir.join("pallets");
    for entry in fs::read_dir(pallets_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            dirs.insert(path.file_name().unwrap().to_str().unwrap().to_string(), path);
        }
    }
    Ok(dirs)
}

pub(crate) fn load_definitions<P: AsRef<Path>>(substrate_dir: &Option<P>) -> std::io::Result<Definitions> {
    let dir = match substrate_dir {
        None => std::env::current_dir()?,
        Some(ref substrate_dir) => PathBuf::from(substrate_dir.as_ref()),
    };
    let config_path = dir.join("hookpoints.json");
    let content = fs::read_to_string(config_path)?;
    let definitions: Definitions = serde_json::from_str(&content)?;
    Ok(definitions)
}