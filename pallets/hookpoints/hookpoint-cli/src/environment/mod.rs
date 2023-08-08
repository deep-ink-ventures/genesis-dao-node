use std::path::PathBuf;

mod substrate;

pub struct Substrate {
    root_folder: PathBuf
}

impl Substrate {
    pub fn new(root: Option<String>) -> Self {
        let root_folder = match root {
            Some(root) => PathBuf::from(root),
            None => std::env::current_dir().unwrap()
        };
    }

}