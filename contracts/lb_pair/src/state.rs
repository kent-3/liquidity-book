#![allow(unused)] // For beginning only.

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, ContractInfo, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use cw_storage_plus::Map;
use lb_libraries::{
    math::tree_math::TreeUint24,
    oracle_helper::Oracle,
    pair_parameter_helper::PairParameters,
    tokens::TokenType,
    types::Bytes32,
    viewing_keys::ViewingKey,
};
use secret_toolkit::{
    serialization::{Bincode2, Json},
    storage::{AppendStore, Item},
};

pub static CONFIG: Item<State, Bincode2> = Item::new(b"config");
pub static BIN_MAP: Map<u32, Bytes32> = Map::new("bins"); //?
pub static BIN_TREE: Item<TreeUint24, Bincode2> = Item::new(b"bin_tree"); //?
pub static ORACLE: Item<Oracle, Bincode2> = Item::new(b"oracle"); //?
pub static EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

#[cw_serde]
pub struct State {
    pub creator: Addr,
    pub factory: ContractInfo,
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub viewing_key: ViewingKey,
    pub pair_parameters: PairParameters,
    pub reserves: Bytes32,
    pub protocol_fees: Bytes32,
    pub lb_token: ContractInfo,
    pub protocol_fees_recipient: Addr,
}

pub fn ephemeral_storage_w(storage: &mut dyn Storage) -> Singleton<NextTokenKey> {
    singleton(storage, EPHEMERAL_STORAGE_KEY)
}

pub fn ephemeral_storage_r(storage: &dyn Storage) -> ReadonlySingleton<NextTokenKey> {
    singleton_read(storage, EPHEMERAL_STORAGE_KEY)
}

#[cw_serde]
pub struct NextTokenKey {
    pub code_hash: String,
}

// NOTE: I don't think these types are necessary, since we are encoding the values into a U256.
// I wonder how much benefit encoding things into 256-bit numbers has over encoding structs with Bincode2.

// #[cw_serde]
// #[derive(Default)]
// pub struct PairParameters {
//     pub base_factor: u16,
//     pub filter_period: u16,
//     pub decay_period: u16,
//     pub reduction_factor: u16,
//     pub variable_fee_control: u32,
//     pub protocol_share: u16,
//     pub max_volatility_accumulator: u32,
//     pub volatility_accumulator: u32,
//     pub volatility_reference: u32,
//     pub index_reference: u32,
//     pub time_of_last_update: u64,
//     pub oracle_id: u16,
//     pub active_id: u32,
// }

// #[cw_serde]
// #[derive(Default)]
// pub struct Oracle {
//     pub oracle_length: u16,
//     pub cumulative_id: u64,
//     pub cumulative_volatility_accumulator: u64,
//     pub cumulative_bin_crossed: u64,
//     pub sample_lifetime: u8,
//     pub sample_creation_timestamp: u64,
// }
