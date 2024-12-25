use crate::types::NextPairKey;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, ContractInfo, Uint128};
use lb_interfaces::{
    lb_factory::ContractImplementation,
    lb_pair::{LbPair, LbPairInformation},
};
use lb_libraries::pair_parameter_helper::PairParameters;
use secret_toolkit::storage::{AppendStore, Item, Keymap, Keyset};
use shade_protocol::{
    // secret_storage_plus::{AppendStore, Item, Map},
    swap::core::TokenType,
    Contract,
};
use std::collections::HashSet;

// TODO: unify const vs static. use secret-toolkit storage types?

pub const CONTRACT_STATUS: Item<ContractStatus> = Item::new(b"contract_status");
pub const STATE: Item<State> = Item::new(b"state");
pub const FLASH_LOAN_FEE: Item<Uint128> = Item::new(b"flashloan_fee");

pub static ALL_LB_PAIRS: AppendStore<LbPair> = AppendStore::new(b"all_lb_pairs");

/// Mapping from a (tokenA, tokenB, binStep) to a LBPair. The tokens are ordered to save gas, but they can be
/// in the reverse order in the actual pair.
/// Always query one of the 2 tokens of the pair to assert the order of the 2 tokens.
pub const LB_PAIRS_INFO: Keymap<(String, String, u16), LbPairInformation> =
    Keymap::new(b"lb_pairs_info");

// TODO: is this necessary? it's not in the original
pub const PRESET_HASHSET: Item<HashSet<u16>> = Item::new(b"preset_hashset");

// TODO: I think we need the secret-toolkit Keymap to be able to iterate over the keys, to avoid
// needing the PRESET_HASHSET.
pub const PRESETS: Keymap<u16, PairParameters> = Keymap::new(b"presets");
// TODO: Would a HashSet would be better for this?
pub static QUOTE_ASSET_WHITELIST: AppendStore<TokenType> =
    AppendStore::new(b"quote_asset_whitelist");

// TODO: is this good?
// The Hashset<u16> will represent the "EnumerableSet.UintSet" from solidity.
// "The primary purpose of EnumerableSet.UintSet is to provide a convenient way to store, iterate, and retrieve
// elements in a set, while ensuring that they remain unique."

/// Mapping from a (tokenA, tokenB) to a set of available bin steps, this is used to keep track of the
/// bin steps that are already used for a pair.
/// The tokens are ordered to save gas, but they can be in the reverse order in the actual pair.
/// Always query one of the 2 tokens of the pair to assert the order of the 2 tokens.
pub const AVAILABLE_LB_PAIR_BIN_STEPS: Keymap<(String, String), HashSet<u16>> =
    Keymap::new(b"available_lb_pair_bin_steps");

#[cw_serde]
pub enum ContractStatus {
    Active,    // allows all operations
    FreezeAll, // blocks everything except admin-protected config changes
}

// TODO: decide on calling this config or state
#[cw_serde]
pub struct State {
    pub contract_info: ContractInfo,
    pub owner: Addr,
    // TODO: I think these should be stored with separate keys.
    pub fee_recipient: Addr,
    pub lb_pair_implementation: ContractImplementation,
    pub lb_token_implementation: ContractImplementation,
    // TODO: change to ContractInfo, or maybe get rid of these auth contracts...
    pub admin_auth: Contract,
    pub query_auth: Contract,
}

pub const EPHEMERAL_STORAGE: Item<NextPairKey> = Item::new(b"ephemeral_storage");
