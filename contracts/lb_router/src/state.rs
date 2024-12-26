use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CanonicalAddr, ContractInfo, Uint128};
use lb_interfaces::lb_factory::ILbFactory;
use secret_toolkit::storage::Item;
use shade_protocol::contract_interfaces::swap::core::TokenType;

pub const FACTORY: Item<ILbFactory> = Item::new(b"factory");

pub const EPHEMERAL_ADD_LIQUIDITY: Item<EphemeralAddLiquidity> =
    Item::new(b"ephemeral_add_liquidity");
pub const EPHEMERAL_REMOVE_LIQUIDITY: Item<EphemeralRemoveLiquidity> =
    Item::new(b"ephemeral_remove_liquidity");
// pub const EPHEMERAL_SWAP_INFO: Item<EphemeralSwapInfo> = Item::new("ephemeral_swap_info");

#[cw_serde]
pub struct EphemeralAddLiquidity {
    pub amount_x_min: Uint128,
    pub amount_y_min: Uint128,
    pub deposit_ids: Vec<u32>,
}

#[cw_serde]
pub struct EphemeralRemoveLiquidity {
    pub amount_x_min: Uint128,
    pub amount_y_min: Uint128,
    pub is_wrong_order: bool,
}

// TODO: not sure if this is necessary...
// #[cw_serde]
// pub struct EphemeralSwapInfo {
//     pub amount: TokenAmount,
//     pub amount_out_min: Option<Uint128>,
//     pub path: Vec<Hop>,
//     pub recipient: Addr,
//     pub current_index: u32,
//     //The next token that will be in the hop
//     pub next_token_in: TokenType,
// }
