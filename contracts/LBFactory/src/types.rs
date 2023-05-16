use cosmwasm_schema::cw_serde;


use libraries::tokens::TokenType;
pub use libraries::types::{LBPair, LBPairInformation};

// TODO: maybe we don't need this file at all?

#[cw_serde]
pub struct NextPairKey {
    pub token_a: TokenType,
    pub token_b: TokenType,
    pub bin_step: u16,
    pub code_hash: String,
}
