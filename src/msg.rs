use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub start_height: u32,
    pub min_difficulty_bits: u32,
    pub min_update_length: u32,
    pub start_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // Each block header should be a 160 character hex string (80 bytes),
    // with the following concatenated together in order in little endian format:
    // 1) ver: i32
    // 2) prev_block: U256
    // 3) mrkl_root: U256
    // 4) time: u32
    // 5) bits: u32
    // 6) nonce: u32
    UpdateBlockOffset { block_headers: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetContractInfo returns the current offset, current hash, start height, and difficulty
    GetContractInfo {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    //u32
    pub start_height: u32,
    // U256
    pub min_difficulty: String,
    // U256
    pub curr_hash: String,
    pub curr_offset: u32,
}
