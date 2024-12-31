use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use liquidity_book::interfaces::{lb_factory::ILbFactory, lb_pair::ILbPair, lb_router::Version};
use secret_toolkit::{serialization::Json, storage::Item};
use shade_protocol::contract_interfaces::swap::core::TokenType;

pub const FACTORY_V2_2: Item<ILbFactory> = Item::new(b"factory_v2_2");

pub const EPHEMERAL_ADD_LIQUIDITY: Item<EphemeralAddLiquidity> =
    Item::new(b"ephemeral_add_liquidity");
pub const EPHEMERAL_REMOVE_LIQUIDITY: Item<EphemeralRemoveLiquidity> =
    Item::new(b"ephemeral_remove_liquidity");
pub const EPHEMERAL_SWAP: Item<EphemeralSwap, Json> = Item::new(b"ephemeral_swap");
pub const EPHEMERAL_SWAP_FOR_EXACT: Item<EphemeralSwapForExact, Json> =
    Item::new(b"ephemeral_swap_for_exact");

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

#[cw_serde]
pub struct EphemeralSwap {
    pub amount_in: Uint128, // updates each loop
    pub amount_out_min: Uint128,
    pub pairs: Vec<ILbPair>,
    pub versions: Vec<Version>,
    pub token_path: Vec<TokenType>,
    pub position: u32,    // updates each loop
    pub swap_for_y: bool, // updates each loop
    pub to: Addr,         // the final swap output recipient
}

#[cw_serde]
pub struct EphemeralSwapForExact {
    pub amounts_in: Vec<Uint128>, // calculated at the start
    pub amount_out: Uint128,
    pub pairs: Vec<ILbPair>,
    pub versions: Vec<Version>,
    pub token_path: Vec<TokenType>,
    pub position: u32,    // updates each loop
    pub swap_for_y: bool, // updates each loop
    pub to: Addr,         // the final swap output recipient
}
