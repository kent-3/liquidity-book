use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CanonicalAddr, ContractInfo};

use secret_toolkit::storage::Item;

pub static CONFIG: Item<Config> = Item::new(b"config");

#[cw_serde]
pub struct Config {
    pub factory: ContractInfo,
    pub admins: Vec<CanonicalAddr>,
}
