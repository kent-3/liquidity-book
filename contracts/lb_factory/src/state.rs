#![allow(unused)] // For beginning only.

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, ContractInfo, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use cw_storage_plus::{Item, Map};
use secret_toolkit::storage::AppendStore;

use lb_libraries::{
    pair_parameter_helper::PairParameters,
    tokens::TokenType,
    types::{Bytes32, ContractInstantiationInfo},
};

use crate::{
    prelude::*,
    types::{LBPair, LBPairInformation, NextPairKey},
};

pub static CONFIG: Item<Config> = Item::new("config");
pub static EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

// TODO: not sure if this should be a Keyset or a Vec.
// pub static ALL_LB_PAIRS: Item<Vec<LBPair>> = Item::new(b"all_lb_pairs");
pub static ALL_LB_PAIRS: AppendStore<LBPair> = AppendStore::new(b"all_lb_pairs");

/// Mapping from a (tokenA, tokenB, binStep) to a LBPair.
/// The tokens are ordered to save gas, but they can be in the reverse order in the actual pair.
pub static LB_PAIRS_INFO: Map<(String, String, u16), LBPairInformation> = Map::new("lb_pairs_info");

/// Map of bin_step to preset, which is an encoded Bytes32 set of pair parameters
pub static PRESETS: Map<u16, PairParameters> = Map::new("presets");

// TODO: not sure if this should be a Keyset or a Vec.
// Does it need to store ContractInfo or would Addr be enough?
// pub static QUOTE_ASSET_WHITELIST: Item<Vec<ContractInfo>> = Item::new(b"quote_asset_whitelist");
pub static QUOTE_ASSET_WHITELIST: AppendStore<TokenType> =
    AppendStore::new(b"quote_asset_whitelist");

/// Mapping from a (tokenA, tokenB) to a set of available bin steps, this is used to keep track of the
/// bin steps that are already used for a pair.
/// The tokens are ordered to save gas, but they can be in the reverse order in the actual pair.
///
// The Vec<u16> will represent the "EnumerableSet.UintSet" from the solidity code.
// The primary purpose of EnumerableSet.UintSet is to provide a convenient way to store, iterate, and retrieve elements in a set, while ensuring that they remain unique.
// TODO: There doesn't appear to be a way to have a mapping to a Keyset, so a Vec will have to do for now...
pub static AVAILABLE_LB_PAIR_BIN_STEPS: Map<(String, String), Vec<u16>> =
    Map::new("available_lb_pair_bin_steps");

// TODO: Rename State to Config?
#[cw_serde]
pub struct Config {
    pub contract_info: ContractInfo,
    pub owner: Addr,
    pub fee_recipient: Addr,
    pub flash_loan_fee: u8,
    pub lb_pair_implementation: ContractInstantiationInfo,
    pub lb_token_implementation: ContractInstantiationInfo,
}

pub fn ephemeral_storage_w(storage: &mut dyn Storage) -> Singleton<NextPairKey> {
    singleton(storage, EPHEMERAL_STORAGE_KEY)
}

pub fn ephemeral_storage_r(storage: &dyn Storage) -> ReadonlySingleton<NextPairKey> {
    singleton_read(storage, EPHEMERAL_STORAGE_KEY)
}
