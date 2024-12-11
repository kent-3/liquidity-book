use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CanonicalAddr, ContractInfo, Uint128};

use shade_protocol::contract_interfaces::swap::core::TokenType;
use shade_protocol::secret_storage_plus::Item;

// TODO: Should I use secret-toolkit-storage instead?

pub const FACTORY: Item<ContractInfo> = Item::new("factory");
// pub const EPHEMERAL_STORAGE: Item<CurrentSwapInfo> = Item::new("ephemeral_storage");

#[cw_serde]
pub struct Config {
    pub factory: ContractInfo,
    // pub admins: Vec<CanonicalAddr>,
    pub viewing_key: String,
}

// #[cw_serde]
// pub struct CurrentSwapInfo {
//     pub(crate) amount: TokenAmount,
//     pub amount_out_min: Option<Uint128>,
//     pub path: Vec<Hop>,
//     pub recipient: Addr,
//     pub current_index: u32,
//     //The next token that will be in the hop
//     pub next_token_in: TokenType,
// }
