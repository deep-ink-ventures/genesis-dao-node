use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Definitions {
    pub name: String,
    pub ink_dependencies: InkDependencies,
    pub pallets: std::collections::HashMap<String, Vec<PalletFunction>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InkDependencies {
    pub ink_version: String,
    pub ink_primitives_version: String,
    pub scale_version: String,
    pub scale_info_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PalletFunction {
    pub hook_point: String,
    pub arguments: Vec<FunctionArgument>,
    pub returns: Option<ReturnValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReturnValue {
    pub default: String,
    #[serde(rename = "type")]
    pub type_: String,
}

impl Definitions {
    pub fn new(name: String, pallets: std::collections::HashMap<String, Vec<PalletFunction>>) -> Self {
        Definitions {
            name,
            pallets,
            ink_dependencies: InkDependencies::default(),
        }
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, substrate_dir: &Option<P>) {
        let dir = match substrate_dir {
            None => std::env::current_dir().unwrap(),
            Some(dir) => PathBuf::from(dir.as_ref()),
        };
        let config_path = dir.join("hookpoints.json");
        let content = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(config_path, content).unwrap();
    }

    pub fn add_pallet_function(&mut self, pallet_name: String, pallet_function: PalletFunction) {
        if let Some(pallet_functions) = self.pallets.get_mut(&pallet_name) {
            pallet_functions.push(pallet_function);
        } else {
            self.pallets.insert(pallet_name, vec![pallet_function]);
        }
    }

     pub fn extract_types(&self) -> Vec<String> {
        let mut types = Vec::new();

        for functions in self.pallets.values() {
            for func in functions {
                for arg in &func.arguments {
                    types.push(arg.type_.clone());
                }

                if let Some(ret_val) = &func.returns {
                    types.push(ret_val.type_.clone());
                }
            }
        }

        types
    }

    pub fn contains_type(&self, target: &[&str]) -> bool {
        let types = self.extract_types();
        for t in types {
            if target.contains(&t.as_str()) {
                return true;
            }
        }
        false
    }
}

impl InkDependencies {
    pub fn default() -> Self {
        InkDependencies {
            ink_version: "4.2".to_string(),
            ink_primitives_version: "4.2".to_string(),
            scale_version: "3".to_string(),
            scale_info_version: "2.6".to_string(),
        }
    }
}
