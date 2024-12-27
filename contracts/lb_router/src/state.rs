use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use lb_interfaces::{lb_factory::ILbFactory, lb_pair::ILbPair, lb_router::Version};
use secret_toolkit::storage::Item;
use shade_protocol::contract_interfaces::swap::core::TokenType;

pub const FACTORY: Item<ILbFactory> = Item::new(b"factory");

pub const EPHEMERAL_ADD_LIQUIDITY: Item<EphemeralAddLiquidity> =
    Item::new(b"ephemeral_add_liquidity");
pub const EPHEMERAL_REMOVE_LIQUIDITY: Item<EphemeralRemoveLiquidity> =
    Item::new(b"ephemeral_remove_liquidity");
pub const EPHEMERAL_SWAP: Item<EphemeralSwap> = Item::new(b"ephemeral_swap");

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
    pub amount_in: Uint128,
    pub amount_out_min: Uint128,
    pub pairs: Vec<ILbPair>,
    pub versions: Vec<Version>,
    pub token_path: Vec<TokenType>,
    pub position: u32,
    pub token_next: TokenType,
    pub swap_for_y: bool,
    pub to: Addr, // the final swap output recipient
}
