use super::{LB_PAIRS_INFO, OFFSET_IS_PRESET_OPEN};
use crate::{Error, Result};
use cosmwasm_std::Deps;
use liquidity_book::{
    interfaces::lb_factory::LbPairInformation,
    libraries::{Bytes32, Encoded},
};
use shade_protocol::swap::core::TokenType;

/// Returns the LBPairInformation if it exists,
/// if not, then the address 0 is returned. The order doesn't matter
pub fn _get_lb_pair_information(
    deps: Deps,
    token_a: &TokenType,
    token_b: &TokenType,
    bin_step: u16,
) -> Option<LbPairInformation> {
    let (token_a, token_b) = _sort_tokens(token_a.clone(), token_b.clone());

    LB_PAIRS_INFO.get(
        deps.storage,
        &(token_a.unique_key(), token_b.unique_key(), bin_step),
    )
}

/// Function to sort 2 tokens in ascending order.
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
