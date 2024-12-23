use cosmwasm_schema::cw_serde;
use shade_protocol::swap::core::TokenType;

// TODO: hmm

#[cw_serde]
pub struct NextPairKey {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub code_hash: String,
    pub is_open: bool,
}
