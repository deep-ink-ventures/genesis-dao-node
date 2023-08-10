use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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