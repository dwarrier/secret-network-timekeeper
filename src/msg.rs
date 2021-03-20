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
    UpdateBlockOffset { blocks: Vec<BlockHeader> }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BlockHeader {
    // Passed as hex strings.
    // i32
    pub ver: String,
    // U256
    pub prev_block: String,
    // U256
    pub mrkl_root: String,
    // u32
    pub time: String,
    // u32
    pub bits: String,
    // u32
    pub nonce: String,
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
    pub curr_offset: u32
}
