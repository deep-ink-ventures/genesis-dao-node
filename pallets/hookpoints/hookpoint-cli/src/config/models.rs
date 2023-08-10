use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Definitions {
    pub(crate) name: String,
    pub(crate) ink_dependencies: InkDependencies,
    pub(crate) pallets: std::collections::HashMap<String, Vec<PalletFunction>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InkDependencies {
    pub(crate) ink_version: String,
    pub(crate) ink_primitives_version: String,
    pub(crate) scale_version: String,
    pub(crate) scale_info_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PalletFunction {
    pub(crate) hook_point: String,
    pub(crate) arguments: Vec<FunctionArgument>,
    pub(crate) returns: Option<ReturnValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) type_: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReturnValue {
    pub(crate) default: String,
    #[serde(rename = "type")]
    pub(crate) type_: String,
}

impl Definitions {
    pub(crate) fn new(name: String, pallets: std::collections::HashMap<String, Vec<PalletFunction>>) -> Self {
        Definitions {
            name,
            pallets,
            ink_dependencies: InkDependencies::default(),
        }
    }

    pub(crate) fn write_to_file<P: AsRef<Path>>(&self, substrate_dir: &Option<P>) {
        let dir = match substrate_dir {
            None => std::env::current_dir().unwrap(),
            Some(dir) => PathBuf::from(dir.as_ref()),
        };
        let config_path = dir.join("hookpoints.json");
        let content = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write(config_path, content).unwrap();
    }

    pub(crate) fn add_pallet_function(&mut self, pallet_name: String, pallet_function: PalletFunction) {
        if let Some(pallet_functions) = self.pallets.get_mut(&pallet_name) {
            pallet_functions.push(pallet_function);
        } else {
            self.pallets.insert(pallet_name, vec![pallet_function]);
        }
    }
}

impl InkDependencies {
    pub(crate) fn default() -> Self {
        InkDependencies {
            ink_version: "4.2".to_string(),
            ink_primitives_version: "4.2".to_string(),
            scale_version: "3".to_string(),
            scale_info_version: "2.6".to_string(),
        }
    }
}
