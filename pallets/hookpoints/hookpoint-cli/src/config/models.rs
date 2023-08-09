use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Definitions {
    pub(crate) name: String,
    pub(crate) pallets: std::collections::HashMap<String, Vec<PalletFunction>>,
    pub(crate) config: Config,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PalletFunction {
    pub(crate) hook_point: String,
    pub(crate) arguments: Vec<FunctionArgument>,
    pub(crate) returns: Option<ReturnValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionArgument {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) type_: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReturnValue {
    pub(crate) default: String,
    #[serde(rename = "type")]
    pub(crate) type_: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub(crate) root_folder: String,
}

impl Definitions {
    pub(crate) fn new(name: String, pallets: std::collections::HashMap<String, Vec<PalletFunction>>, config: Config) -> Self {
        Definitions {
            name,
            pallets,
            config,
        }
    }

    pub(crate) fn write_to_file(&self) {
        let content = serde_json::to_string_pretty(&self).unwrap();
        std::fs::write("hookpoints.json", content).unwrap();
    }


    pub(crate) fn add_pallet_function(&mut self, pallet_name: String, pallet_function: PalletFunction) {
        if let Some(pallet_functions) = self.pallets.get_mut(&pallet_name) {
            pallet_functions.push(pallet_function);
        } else {
            self.pallets.insert(pallet_name, vec![pallet_function]);
        }
    }
}

impl Config {
    pub(crate) fn new(root_folder: String) -> Self {
        Config {
            root_folder
        }
    }
}