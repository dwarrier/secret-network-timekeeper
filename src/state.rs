use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use primitive_types::U256;

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    // The height of the start block hash.
    pub start_height: u32,
    // Number of valid blocks that have been seen from the start height.
    pub curr_offset: u32,
    // The hash of the current block.
    pub curr_hash: String,
    // The difficulty of any block cannot be greater than this value during validation.
    // Big endian hex representation of a U256, since that type isn't serializable.
    pub threshold_difficulty: String,
    // When updating, must pass in this many blocks or more.
    // Intended to increase the amount of work for creating invalid chains.
    pub min_update_length: u32,
    pub owner: CanonicalAddr,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}
