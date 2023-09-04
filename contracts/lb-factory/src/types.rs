use cosmwasm_schema::cw_serde;

use lb_libraries::tokens::TokenType;
pub use lb_libraries::types::{LBPair, LBPairInformation};

// TODO: maybe we don't need this file at all?

#[cw_serde]
pub struct NextPairKey {
    pub token_a: TokenType,
    pub token_b: TokenType,
    pub bin_step: u16,
    pub code_hash: String,
}
