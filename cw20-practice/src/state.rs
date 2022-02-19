use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Constants {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}
