use super::{state::*, OFFSET_IS_PRESET_OPEN};
use crate::prelude::*;
use cosmwasm_std::Deps;
use lb_interfaces::lb_pair::LbPairInformation;
use lb_libraries::{math::encoded::Encoded, types::Bytes32};
use shade_protocol::swap::core::TokenType;

/// Returns the LbPairInformation if it exists, if not, then the address 0 is returned. The order doesn't matter
///
/// # Arguments
///
/// * `token_a` - The address of the first token of the pair
/// * `token_b` - The address of the second token of the pair
/// * `bin_step` - The bin step of the LbPair
///
/// # Returns
///
/// * The LbPairInformation
pub fn _get_lb_pair_information(
    deps: Deps,
    token_a: TokenType,
    token_b: TokenType,
    bin_step: u16,
) -> Result<LbPairInformation> {
    let (token_a, token_b) = _sort_tokens(token_a, token_b);
    let info = LB_PAIRS_INFO
        .load(
            deps.storage,
            (token_a.unique_key(), token_b.unique_key(), bin_step),
        )
        // FIXME: return the LbPairNotCreated error instead of unwrapping!
        .unwrap();

    Ok(info)
}

/// Function to sort 2 tokens in ascending order.
///
/// # Arguments
///
/// * `token_a` - The first token
/// * `token_b` - The second token
///
/// # Returns
///
/// * The sorted first token
/// * The sorted second token
pub fn _sort_tokens(token_a: TokenType, token_b: TokenType) -> (TokenType, TokenType) {
    if token_a.unique_key() < token_b.unique_key() {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    }
}

pub fn _is_preset_open(preset: Bytes32) -> bool {
    preset.decode_bool(OFFSET_IS_PRESET_OPEN)
}
