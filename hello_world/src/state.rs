use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use cosmwasm_std::Storage;

pub static NAME_KEY: &[u8] = b"name";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub name: String,
}

pub fn writable_name(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, NAME_KEY)
}

pub fn name_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, NAME_KEY)
}
