use std::fs;
use std::collections::HashMap;
use std::path::{PathBuf};

pub struct Substrate {
    pub(crate) pallets: HashMap<String, PathBuf>
}

impl Substrate {
    pub fn new(root: Option<&String>) -> Self {
        let mut root_folder = match root {
            Some(root) =>  PathBuf::from(root),
            None => std::env::current_dir().expect("unable to get current directory")
        };
        Substrate {
            pallets: get_pallets(&mut root_folder).expect("pallets folder not found")
        }
    }
}

fn get_pallets(dir: &mut PathBuf) -> std::io::Result<HashMap<String, PathBuf>> {
    let mut dirs: HashMap<String, PathBuf> = HashMap::new();
    dir.push("pallets");
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            dirs.insert(path.file_name().unwrap().to_str().unwrap().to_string(), path);
        }
    }
    Ok(dirs)
}