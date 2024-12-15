use crate::types::NextPairKey;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, ContractInfo, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use lb_interfaces::{
    lb_factory::ContractImplementation,
    lb_pair::{LbPair, LbPairInformation},
};
use lb_libraries::pair_parameter_helper::PairParameters;
use shade_protocol::{
    secret_storage_plus::{AppendStore, Item, Map},
    swap::core::TokenType,
    Contract,
};
use std::collections::HashSet;
// use secret_toolkit::storage::{AppendStore, Item, Keymap, Keyset};

// TODO: unify const vs static. use secret-toolkit storage types?

pub const CONTRACT_STATUS: Item<ContractStatus> = Item::new("contract_status");
pub const STATE: Item<State> = Item::new("state");
pub const FLASH_LOAN_FEE: Item<Uint128> = Item::new("flashloan_fee");

pub static ALL_LB_PAIRS: AppendStore<LbPair> = AppendStore::new("all_lb_pairs");

/// Mapping from a (tokenA, tokenB, binStep) to a LBPair. The tokens are ordered to save gas, but they can be
/// in the reverse order in the actual pair.
/// Always query one of the 2 tokens of the pair to assert the order of the 2 tokens.
pub const LB_PAIRS_INFO: Map<(String, String, u16), LbPairInformation> = Map::new("lb_pairs_info");

// TODO: is this necessary? it's not in the original
pub const PRESET_HASHSET: Item<HashSet<u16>> = Item::new("preset_hashset");

// TODO: I think we need the secret-toolkit KeyMap to be able to iterate over the keys, to avoid
// needing the PRESET_HASHSET.
pub const PRESETS: Map<u16, PairParameters> = Map::new("presets");
// TODO: Would a HashSet would be better for this?
pub static QUOTE_ASSET_WHITELIST: AppendStore<TokenType> =
    AppendStore::new("quote_asset_whitelist");

// TODO: use a HashSet instead?
// The Vec<u16> will represent the "EnumerableSet.UintSet" from the solidity code.
// The primary purpose of EnumerableSet.UintSet is to provide a convenient way to store, iterate, and retrieve
// elements in a set, while ensuring that they remain unique.

/// Mapping from a (tokenA, tokenB) to a set of available bin steps, this is used to keep track of the
/// bin steps that are already used for a pair.
/// The tokens are ordered to save gas, but they can be in the reverse order in the actual pair.
/// Always query one of the 2 tokens of the pair to assert the order of the 2 tokens.
pub const AVAILABLE_LB_PAIR_BIN_STEPS: Map<(String, String), HashSet<u16>> =
    Map::new("available_lb_pair_bin_steps");

#[cw_serde]
pub enum ContractStatus {
    Active,    // allows all operations
    FreezeAll, // blocks everything except admin-protected config changes
}

#[cw_serde]
pub struct State {
    pub contract_info: ContractInfo,
    pub owner: Addr,
    pub fee_recipient: Addr,
    pub lb_pair_implementation: ContractImplementation,
    pub lb_token_implementation: ContractImplementation,
    pub admin_auth: Contract,
    pub query_auth: Contract,
}

// TODO: Be consistent with the storage types used. Other contracts use Item.

pub static EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

pub fn ephemeral_storage_w(storage: &mut dyn Storage) -> Singleton<NextPairKey> {
    singleton(storage, EPHEMERAL_STORAGE_KEY)
}

pub fn ephemeral_storage_r(storage: &dyn Storage) -> ReadonlySingleton<NextPairKey> {
    singleton_read(storage, EPHEMERAL_STORAGE_KEY)
}
