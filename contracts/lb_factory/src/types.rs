use cosmwasm_schema::cw_serde;
// TODO: I'm not sure if these types should be in the interface crate or somewhere else...
pub use lb_interfaces::lb_pair::{LbPair, LbPairInformation};
use shade_protocol::swap::core::TokenType;

#[cw_serde]
pub struct NextPairKey {
    pub token_a: TokenType,
    pub token_b: TokenType,
    pub bin_step: u16,
    pub code_hash: String,
    pub is_open: bool,
}
