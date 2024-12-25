use super::OFFSET_IS_PRESET_OPEN;
use lb_libraries::{Bytes32, Encoded};
use shade_protocol::swap::core::TokenType;

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
