use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, CanonicalAddr, Deps, Response, StdResult, WasmMsg};
use cosmwasm_std::{ContractInfo, StdError};

use crate::Bytes32;
use ethnum::uint;
use ethnum::U256;

pub const BEFORE_SWAP_FLAG: U256 = uint!("1461501637330902918203684832716283019655932542976"); // 1 << 160
pub const AFTER_SWAP_FLAG: U256 = uint!("2923003274661805836407369665432566039311865085952"); // 1 << 161
pub const BEFORE_FLASH_LOAN_FLAG: U256 = uint!("5846006549323611672814739330865132078623730171904"); // 1 << 162
pub const AFTER_FLASH_LOAN_FLAG: U256 = uint!("11692013098647223345629478661730264157247460343808"); // 1 << 163
pub const BEFORE_MINT_FLAG: U256 = uint!("23384026197294446691258957323460528314494920687616"); // 1 << 164
pub const AFTER_MINT_FLAG: U256 = uint!("46768052394588893382517914646921056628989841375232"); // 1 << 165
pub const BEFORE_BURN_FLAG: U256 = uint!("93536104789177786765035829293842113257979682750464"); // 1 << 166
pub const AFTER_BURN_FLAG: U256 = uint!("187072209578355573530071658587684226515959365500928"); // 1 << 167
pub const BEFORE_TRANSFER_FLAG: U256 = uint!("374144419156711147060143317175368453031918731001856"); // 1 << 168
pub const AFTER_TRANSFER_FLAG: U256 = uint!("748288838313422294120286634350736906063837462003712"); // 1 << 169

// we are forced to store an extra Bytes32 for the code hash, to call the Hooks contract
#[cw_serde]
pub struct HooksParameters {
    pub hooks_parameters: Bytes32,
    pub code_hash: Bytes32,
}

impl HooksParameters {
    pub fn new(hooks_parameters: Bytes32, code_hash: &str) -> Self {
        let code_hash = code_hash_to_bytes32(code_hash).expect("invalid code hash length");
        HooksParameters {
            hooks_parameters,
            code_hash,
        }
    }
}

/// Converts a hex-encoded code hash (SHA-256) into a Bytes32 array.
/// The input should be a 64-character hex string.
pub fn code_hash_to_bytes32(code_hash: &str) -> StdResult<Bytes32> {
    // Ensure the hex string is 64 characters (32 bytes)
    if code_hash.len() != 64 {
        return Err(StdError::generic_err(
            "Invalid code_hash length. Must be 64 characters.",
        ));
    }

    // Convert the hex string to bytes
    let bytes = hex::decode(code_hash).map_err(|e| StdError::generic_err(e.to_string()))?;

    // Ensure the resulting byte array is exactly 32 bytes
    let bytes32: [u8; 32] = bytes.try_into().expect("code hash must be 32 bytes");

    Ok(bytes32)
}

#[cw_serde]
pub struct Parameters {
    pub hooks: CanonicalAddr,
    pub before_swap: bool,
    pub after_swap: bool,
    pub before_flash_loan: bool,
    pub after_flash_loan: bool,
    pub before_mint: bool,
    pub after_mint: bool,
    pub before_burn: bool,
    pub after_burn: bool,
    pub before_batch_transfer_from: bool,
    pub after_batch_transfer_from: bool,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            hooks: CanonicalAddr::from([0u8; 20]),
            before_swap: false,
            after_swap: false,
            before_flash_loan: false,
            after_flash_loan: false,
            before_mint: false,
            after_mint: false,
            before_burn: false,
            after_burn: false,
            before_batch_transfer_from: false,
            after_batch_transfer_from: false,
        }
    }
}

/**
 * @dev Helper function to encode the hooks parameters to a single bytes32 value
 * @param parameters The hooks parameters
 * @return hooksParameters The encoded hooks parameters
 */
pub fn encode(parameters: Parameters) -> Bytes32 {
    let mut buffer = [0u8; 32];

    let canonical_bytes = parameters.hooks.as_slice();
    buffer[12..32].copy_from_slice(canonical_bytes);

    let mut hooks_parameters = U256::from_be_bytes(buffer);
    if parameters.before_swap {
        hooks_parameters |= BEFORE_SWAP_FLAG;
    }
    if parameters.after_swap {
        hooks_parameters |= AFTER_SWAP_FLAG;
    }
    if parameters.before_flash_loan {
        hooks_parameters |= BEFORE_FLASH_LOAN_FLAG;
    }
    if parameters.after_flash_loan {
        hooks_parameters |= AFTER_FLASH_LOAN_FLAG;
    }
    if parameters.before_mint {
        hooks_parameters |= BEFORE_MINT_FLAG;
    }
    if parameters.after_mint {
        hooks_parameters |= AFTER_MINT_FLAG;
    }
    if parameters.before_burn {
        hooks_parameters |= BEFORE_BURN_FLAG;
    }
    if parameters.after_burn {
        hooks_parameters |= AFTER_BURN_FLAG;
    }
    if parameters.before_batch_transfer_from {
        hooks_parameters |= BEFORE_TRANSFER_FLAG;
    }
    if parameters.after_batch_transfer_from {
        hooks_parameters |= AFTER_TRANSFER_FLAG;
    }

    hooks_parameters.to_be_bytes()
}

/**
 * @dev Helper function to decode the hooks parameters from a single bytes32 value
 * @param hooksParameters The encoded hooks parameters
 * @return parameters The hooks parameters
 */
pub fn decode(hooks_parameters: Bytes32) -> Parameters {
    let mut parameters = Parameters::default();

    parameters.hooks = get_hooks(hooks_parameters);

    // Convert to ethnum::U256 to get access to bitwise operations
    let hooks_parameters = U256::from_be_bytes(hooks_parameters);

    parameters.before_swap = (hooks_parameters & BEFORE_SWAP_FLAG) != 0;
    parameters.after_swap = (hooks_parameters & AFTER_SWAP_FLAG) != 0;
    parameters.before_flash_loan = (hooks_parameters & BEFORE_FLASH_LOAN_FLAG) != 0;
    parameters.after_flash_loan = (hooks_parameters & AFTER_FLASH_LOAN_FLAG) != 0;
    parameters.before_mint = (hooks_parameters & BEFORE_MINT_FLAG) != 0;
    parameters.after_mint = (hooks_parameters & AFTER_MINT_FLAG) != 0;
    parameters.before_burn = (hooks_parameters & BEFORE_BURN_FLAG) != 0;
    parameters.after_burn = (hooks_parameters & AFTER_BURN_FLAG) != 0;
    parameters.before_batch_transfer_from = (hooks_parameters & BEFORE_TRANSFER_FLAG) != 0;
    parameters.after_batch_transfer_from = (hooks_parameters & AFTER_TRANSFER_FLAG) != 0;

    parameters
}

/**
 * @dev Helper function to get the hooks address from the encoded hooks parameters
 * @param hooksParameters The encoded hooks parameters
 * @return hooks The hooks address
 */
pub fn get_hooks(hooks_parameters: Bytes32) -> CanonicalAddr {
    // Extract the upper 20 bytes (big-endian)
    let canonical = CanonicalAddr::from(&hooks_parameters[12..32]);

    canonical

    // Humanize the canonical address
    // deps.api.addr_humanize(&canonical).unwrap()
}

/**
 * @dev Helper function to set the hooks address in the encoded hooks parameters
 * @param hooksParameters The encoded hooks parameters
 * @param newHooks The new hooks address
 * @return hooksParameters The updated hooks parameters
 */
pub fn set_hooks(mut hooks_parameters: Bytes32, new_hooks: CanonicalAddr) -> Bytes32 {
    hooks_parameters[12..32].copy_from_slice(new_hooks.as_slice());

    hooks_parameters

    // let new_hooks = U256::from_be_bytes(buffer);
    //
    // (U256::from_be_bytes(hooks_parameters) | new_hooks).to_be_bytes()
}

/**
 * @dev Helper function to get the flags from the encoded hooks parameters
 * @param hooksParameters The encoded hooks parameters
 * @return flags The flags
 */
pub fn get_flags(hooks_parameters: Bytes32) -> [u8; 12] {
    let mut flags = [0u8; 12];
    flags.copy_from_slice(&hooks_parameters[..12]);
    flags
}

/**
 * @dev Helper function call the onHooksSet function on the hooks contract, only if the
 * hooksParameters is not 0
 * @param hooksParameters The encoded hooks parameters
 * @param onHooksSetData The data to pass to the onHooksSet function
 */
// NOTE: maybe this isn't necessary after all.
// since we won't be calling the function, just creating a msg
pub fn on_hooks_set(
    deps: Deps,
    hooks_parameters: Bytes32,
    hooks_code_hash: Bytes32,
    on_hooks_set_data: Binary,
) -> Option<WasmMsg> {
    if hooks_parameters != [0u8; 32] {
        // TODO: we get the address out of the hooks_parameters and create an
        // ExecuteMsg::OnHooksSet message... but how are we going to get the code hash!

        let hooks = deps
            .api
            .addr_humanize(&get_hooks(hooks_parameters))
            .unwrap();

        // I don't want to have lb-interfaces as a dependency, but maybe it's not a big deal?
        // WasmMsg::Execute { contract_addr: hooks, code_hash: hex::encode(hooks_code_hash), msg: (), funds: vec![] }

        //     abi.encodeWithSelector(
        //         ILBHooks.onHooksSet.selector,
        //         hooksParameters,
        //         onHooksSetData,
        //     ),

        todo!()
    } else {
        None
    }
}

// TODO: all the rest of this module

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let hooks = CanonicalAddr::from(vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        ]);
        let params = Parameters {
            hooks,
            before_swap: true,
            after_swap: false,
            before_flash_loan: true,
            ..Parameters::default()
        };

        let encoded = encode(params.clone());
        let decoded = decode(encoded);

        assert_eq!(params.hooks, decoded.hooks);
        assert_eq!(params.before_swap, decoded.before_swap);
        assert_eq!(params.before_flash_loan, decoded.before_flash_loan);
    }
}
