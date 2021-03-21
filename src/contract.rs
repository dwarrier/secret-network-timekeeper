use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage,
};

extern crate rustc_hex as hex;
use crate::msg::{HandleMsg, InfoResponse, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};

use primitive_types::U256;
use sha2::{Digest, Sha256};
use hex::{FromHex, ToHex};
use snafu::{Backtrace, GenerateBacktrace, Error};
use std::convert::TryFrom;
use std::num::ParseIntError;

// Represents the length of an 80 byte block header hex string.
const BLOCK_HEADER_LEN: usize = 160;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        curr_hash: msg.start_hash,
        curr_offset: 0,
        start_height: msg.start_height,
        threshold_difficulty: format!("{:x}", bits_to_difficulty(msg.min_difficulty_bits)),
        min_update_length: msg.min_update_length,
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateBlockOffset {
            block_headers: blocks,
        } => try_update_offset(deps, env, blocks),
    }
}

// Double hashes a hex string and returns a hex string.
pub fn double_hash_hex(hex_str: &String) -> String {
    let inp: Vec<u8> = hex_str.from_hex().unwrap();
    let first: [u8; 32] = Sha256::digest(&inp[..]).into();
    let second: [u8; 32] = Sha256::digest(&first).into();
    return second.to_hex();
}

// bits is a u32 as a hex string in little endian format.
pub fn parse_bits(bits: &str) -> Result<u32, ParseIntError> {
    // This will read the value in as big endian.
    let parsed = u32::from_str_radix(bits, 16);
    let parsed = match parsed {
        Ok(num) => num,
        Err(error) => return Result::Err(error),
    };
    // We assume bits was passed in as little endian, so swap to get the actual value.
    parsed.swap_bytes();
    return Result::Ok(parsed);
}

// Convert bits encoding into a difficulty number.
// See https://en.bitcoin.it/wiki/Difficulty.
// From https://bitcoin.stackexchange.com/questions/30467/what-are-the-equations-to-convert-between-bits-and-difficulty.
pub fn bits_to_difficulty(n_compact: u32) -> U256 {
    let n_size: u32 = n_compact >> 24;
    let mut n_word: u32 = n_compact & 0x007fffff;
    let mut diff: U256;
    if (n_size <= 3) {
        n_word >>= 8 * (3 - n_size);
        diff = U256::from(n_word);
    } else {
        diff = U256::from(n_word);
        diff <<= 8 * (n_size - 3);
    }
    diff
}

// Convenience function to go from little to big endian for a even length hex string.
pub fn flip_bytes_in_str(hex_str: &String) -> String {
    let mut inp: Vec<u8> = hex_str.from_hex().unwrap();
    inp.reverse();
    inp.to_hex()
}

// Verifies header values. If successful, updates the offset
// and the current block header hash.
pub fn try_update_offset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    headers: Vec<String>,
) -> StdResult<HandleResponse> {
    config(&mut deps.storage).update(|mut state| {

        // Check that the number of block header hashes passed in is large enough.
        let num_headers = u32::try_from(headers.len()).unwrap();
        if state.min_update_length > num_headers {
            return Err(StdError::GenericErr {
                msg: format!(
                    "Number of blocks provided ({}) is less than minimum required ({})",
                    num_headers, state.min_update_length
                ),
                backtrace: Option::Some(Backtrace::generate()),
            });
        }

        // The first header must reference the current hash stored by the contract.
        let mut prev_hash = state.curr_hash;

        // Verify every header.
        for header in headers.iter() {
            // Check the header length.
            if header.len() != BLOCK_HEADER_LEN {
                return Err(StdError::GenericErr {
                    msg: format!(
                        "Encoded block header length is {}, must be {}",
                        header.len(),
                        BLOCK_HEADER_LEN
                    ),
                    backtrace: Option::Some(Backtrace::generate()),
                });
            }

            // Check the difficulty bits in the header against the
            // difficulty threshold stored by the contract.
            let difficulty_bits = &header[144..144 + 8];
            // TODO: handle error
            let parsed = parse_bits(difficulty_bits).unwrap();
            let block_diff = bits_to_difficulty(parsed.swap_bytes());
            // TODO: handle error here
            let thresh_diff = U256::from_str_radix(&state.threshold_difficulty, 16).unwrap();
            if block_diff > thresh_diff {
                return Err(StdError::GenericErr {
                    msg: format!(
                        "Block difficulty {} can't be greater than threshold {}",
                        format!("{:x}", block_diff),
                        format!("{:x}", thresh_diff)
                    ),
                    backtrace: Option::Some(Backtrace::generate()),
                });
            }

            // Check that the header references the correct previous header hash.
            let prev_block = &header[8..8 + 64];
            if prev_block != prev_hash {
                return Err(StdError::GenericErr {
                    msg: format!(
                        "Previous block header hash {} is not equal to value in header {}",
                        prev_hash, prev_block
                    ),
                    backtrace: Option::Some(Backtrace::generate()),
                });
            }

            // Compute the target hash and update the prev_hash with it.
            prev_hash = double_hash_hex(&header);

            // Check the difficulty of the target hash against the block difficulty.
            let flipped = flip_bytes_in_str(&prev_hash);
            // TODO: handle error here
            let t = U256::from_str_radix(&flipped, 16).unwrap();
            if t > block_diff {
                return Err(StdError::GenericErr {
                    msg: format!(
                        "Block header hash {} must be less than block difficulty {}",
                        format!("{:x}", t),
                        format!("{:x}", block_diff)
                    ),
                    backtrace: Option::Some(Backtrace::generate()),
                });
            }
        }

        state.curr_hash = prev_hash;
        state.curr_offset += num_headers;
        Ok(state)
    })?;

    // TODO: what is this for?
    Ok(HandleResponse::default())
}

// Returns hash of the last block if the chain is valid
/*
fn verify_blocks(
    blocks: Vec<String>,
    min_update_length: u32,
    min_difficulty_bits: u32,
    curr_hash: U256,
    curr_offset: u32
) -> Result<Option<String>, dyn Error> {

}

 */

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetContractInfo {} => to_binary(&query_info(deps)?),
    }
}

fn query_info<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<InfoResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(InfoResponse {
        start_height: state.start_height,
        min_difficulty: state.threshold_difficulty,
        curr_hash: state.curr_hash,
        curr_offset: state.curr_offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{coins, from_binary, StdError};
    use std::io::Empty;

    fn default_init_msg() -> InitMsg {
        InitMsg {
            start_height: 10,
            min_difficulty_bits: 0x1b0404cbu32,
            start_hash: "81cd02ab7e569e8bcd9317e2fe99f2de44d49ab2b8851ba4a308000000000000"
                .parse()
                .unwrap(),
            min_update_length: 3,
        }
    }

    fn test_block_headers() -> Vec<String> {
        // Values are encoded as hex strings in little endian format.
       vec![
            [
                // version
                "01000000",
                // previous block hash
                "81cd02ab7e569e8bcd9317e2fe99f2de44d49ab2b8851ba4a308000000000000",
                // merkle root hash
                "e320b6c2fffc8d750423db8b1eb942ae710e951ed797f7affc8892b0f1fc122b",
                // unix timestamp
                "c7f5d74d",
                // difficulty bits
                "f2b9441a",
                // nonce
                "42a14695",
            ]
            .concat(),
            [
                "01000000",
                "1dbd981fe6985776b644b173a4d0385ddc1aa2a829688d1e0000000000000000",
                "b371c14921b20c2895ed76545c116e0ad70167c5c4952ca201f5d544a26efb53",
                "b4f6d74d",
                "f2b9441a",
                "071a0c81",
            ]
            .concat(),
            [
                "01000000",
                "85afcb448a3fcde31dc78babd352d9dbde6fcb566777ea33051c000000000000",
                "ca5b6b96fe65e1a7d50e7c3025a176472ba26d44512de86a6f3e39649330cd2f",
                "16f7d74d",
                "f2b9441a",
                "8574adaf",
            ]
            .concat(),
        ]
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        // The start_hash is a little endian hex string.
        let start_hash: String = "81cd02ab7e569e8bcd9317e2fe99f2de44d49ab2b8851ba4a308000000000000"
            .parse()
            .unwrap();
        let min_bits = bits_to_difficulty(0x1b0404cbu32);

        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, default_init_msg()).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetContractInfo {}).unwrap();
        let value: InfoResponse = from_binary(&res).unwrap();
        assert_eq!(start_hash, value.curr_hash);
        assert_eq!(0, value.curr_offset);
        assert_eq!(10, value.start_height);
        assert_eq!(format!("{:x}", min_bits), value.min_difficulty);
    }

    #[test]
    fn double_hash_test() {
        // See https://en.bitcoin.it/wiki/Block_hashing_algorithm.
        let a = "01000000";
        let b = "81cd02ab7e569e8bcd9317e2fe99f2de44d49ab2b8851ba4a308000000000000";
        let c = "e320b6c2fffc8d750423db8b1eb942ae710e951ed797f7affc8892b0f1fc122b";
        let d = "c7f5d74d";
        let e = "f2b9441a";
        let f = "42a14695";
        let result: String = [a, b, c, d, e, f].join("");
        let hashed = double_hash_hex(&result);
        assert_eq!(
            hashed,
            "1dbd981fe6985776b644b173a4d0385ddc1aa2a829688d1e0000000000000000"
        )
    }

    /*
    #[test]
    fn parse_bits_test() {}

    #[test]
    fn bits_to_difficulty_test() {}
     */

    #[test]
    fn update() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, default_init_msg()).unwrap();

        // anyone can increment
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::UpdateBlockOffset {
            block_headers: test_block_headers(),
        };
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase offset by 3
        let res = query(&deps, QueryMsg::GetContractInfo {}).unwrap();
        let value: InfoResponse = from_binary(&res).unwrap();
        assert_eq!(3, value.curr_offset);
        // big-endian: 0000000000001112dff6e2a85b35d4f7ab7005b1b749282eeb1fdf094722601e
        assert_eq!(
            "1e60224709df1feb2e2849b7b10570abf7d4355ba8e2f6df1211000000000000",
            value.curr_hash
        );
    }

    #[test]
    fn min_difficulty_enforced() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let msg = InitMsg {
            start_height: 10,
            min_difficulty_bits: 0x1a44b9f1u32,
            start_hash: "81cd02ab7e569e8bcd9317e2fe99f2de44d49ab2b8851ba4a308000000000000"
                .parse()
                .unwrap(),
            min_update_length: 3,
        };

        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can increment
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::UpdateBlockOffset {
            block_headers: test_block_headers(),
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::GenericErr { msg, backtrace }) => {
                assert_eq!(msg, "Block difficulty 44b9f20000000000000000000000000000000000000000000000 can't be greater than threshold 44b9f10000000000000000000000000000000000000000000000");
            }
            _ => panic!("Must return an error"),
        }
    }

    #[test]
    fn min_num_blocks_enforced() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, default_init_msg()).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::UpdateBlockOffset {
            block_headers: test_block_headers().split_last().unwrap().1.to_vec(),
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::GenericErr { msg, backtrace }) => {
                assert_eq!(
                    msg,
                    "Number of blocks provided (2) is less than minimum required (3)"
                );
            }
            _ => panic!("Must return an error"),
        }
    }

    #[test]
    fn bad_headers() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, default_init_msg()).unwrap();

        let env = mock_env("anyone", &coins(2, "token"));
        let mut partial_blocks = test_block_headers().split_last().unwrap().1.to_vec();
        partial_blocks.push("bad_header".to_string());
        let msg = HandleMsg::UpdateBlockOffset {
            block_headers: partial_blocks,
        };
        let res = handle(&mut deps, env, msg);
        match res {
            Err(StdError::GenericErr { msg, backtrace }) => {
                assert_eq!(msg, "Encoded block header length is 10, must be 160");
            }
            _ => panic!("Must return an error"),
        }
    }
}
