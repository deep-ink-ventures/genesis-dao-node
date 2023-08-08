use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub(crate) name: String,
    pub(crate) pallets: std::collections::HashMap<String, Vec<PalletFunction>>,
}

#[derive(Deserialize, Debug)]
pub struct PalletFunction {
    pub(crate) hook_point: String,
    pub(crate) arguments: Vec<FunctionArgument>,
    pub(crate) returns: ReturnValue,
}

#[derive(Deserialize, Debug)]
pub struct FunctionArgument {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) type_: String,
}

#[derive(Deserialize, Debug)]
pub struct ReturnValue {
    pub(crate) default: String,
    #[serde(rename = "type")]
    pub(crate) type_: String
}