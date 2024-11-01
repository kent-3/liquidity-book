use crate::types::NextPairKey;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, ContractInfo, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use lb_interfaces::{
    lb_factory::ContractImplementation,
    lb_pair::{LbPair, LbPairInformation, RewardsDistributionAlgorithm},
};
use lb_libraries::pair_parameter_helper::PairParameters;
use shade_protocol::{
    secret_storage_plus::{AppendStore, Item, Map},
    swap::core::TokenType,
    Contract,
};
use std::collections::HashSet;

pub const CONTRACT_STATUS: Item<ContractStatus> = Item::new("contract_status");
pub const STATE: Item<State> = Item::new("state");
pub static EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

// pub static ALL_LB_PAIRS: Item<Vec<LbPair>> = Item::new(b"all_lb_pairs");
pub static ALL_LB_PAIRS: AppendStore<LbPair> = AppendStore::new("all_lb_pairs");

/// Mapping from a (tokenA, tokenB, binStep) to a LbPair.
/// The tokens are ordered to save gas, but they can be in the reverse order in the actual pair.
pub const LB_PAIRS_INFO: Map<(String, String, u16), LbPairInformation> = Map::new("lb_pairs_info");

pub const PRESET_HASHSET: Item<HashSet<u16>> = Item::new("preset_hashset");

/// Map of bin_step to preset, which is an encoded Bytes32 set of pair parameters
pub const PRESETS: Map<u16, PairParameters> = Map::new("presets");

/// Map of bin_step to preset, which is an encoded Bytes32 set of pair parameters
pub const STAKING_PRESETS: Map<u16, StakingPreset> = Map::new("stkaing_presets");

// Does it need to store ContractInfo or would Addr be enough?
// pub static QUOTE_ASSET_WHITELIST: Item<Vec<ContractInfo>> = Item::new(b"quote_asset_whitelist");
pub static QUOTE_ASSET_WHITELIST: AppendStore<TokenType> =
    AppendStore::new("quote_asset_whitelist");

/// Mapping from a (tokenA, tokenB) to a set of available bin steps, this is used to keep track of the
/// bin steps that are already used for a pair.
/// The tokens are ordered to save gas, but they can be in the reverse order in the actual pair.
///
// The Vec<u16> will represent the "EnumerableSet.UintSet" from the solidity code.
// The primary purpose of EnumerableSet.UintSet is to provide a convenient way to store, iterate, and retrieve elements in a set, while ensuring that they remain unique.
pub const AVAILABLE_LB_PAIR_BIN_STEPS: Map<(String, String), Vec<u16>> =
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
    pub staking_contract_implementation: ContractImplementation,
    pub admin_auth: Contract,
    pub query_auth: Contract,
    pub recover_staking_funds_receiver: Addr,
    pub max_bins_per_swap: Option<u32>,
}

#[cw_serde]
pub struct StakingPreset {
    pub total_reward_bins: u32,
    pub rewards_distribution_algorithm: RewardsDistributionAlgorithm,
    pub epoch_staking_index: u64,
    pub epoch_staking_duration: u64,
    pub expiry_staking_duration: Option<u64>,
}

pub fn ephemeral_storage_w(storage: &mut dyn Storage) -> Singleton<NextPairKey> {
    singleton(storage, EPHEMERAL_STORAGE_KEY)
}

pub fn ephemeral_storage_r(storage: &dyn Storage) -> ReadonlySingleton<NextPairKey> {
    singleton_read(storage, EPHEMERAL_STORAGE_KEY)
}
