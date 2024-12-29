use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, ContractInfo, Uint128};
use liquidity_book::{
    interfaces::{
        lb_factory::Implementation,
        lb_pair::{LbPair, LbPairInformation},
    },
    libraries::pair_parameter_helper::PairParameters,
};
use secret_toolkit::{
    serialization::Json,
    storage::{AppendStore, Item, Keymap, Keyset},
};
use shade_protocol::{swap::core::TokenType, Contract};
use std::collections::HashSet;

pub static STATE: Item<State> = Item::new(b"state");
pub static CONTRACT_STATUS: Item<ContractStatus, Json> = Item::new(b"contract_status");

pub static FEE_RECIPIENT: Item<Addr> = Item::new(b"fee_recipient");
pub static FLASH_LOAN_FEE: Item<Uint128> = Item::new(b"flashloan_fee");

pub static LB_PAIR_IMPLEMENTATION: Item<Implementation> = Item::new(b"lb_pair_implementation");
pub static LB_TOKEN_IMPLEMENTATION: Item<Implementation> = Item::new(b"lb_token_implementation");

pub static ALL_LB_PAIRS: AppendStore<LbPair, Json> = AppendStore::new(b"all_lb_pairs");

/// Mapping from a (tokenA, tokenB, binStep) to a LBPair. The tokens are ordered to save gas, but they can be
/// in the reverse order in the actual pair.
/// Always query one of the 2 tokens of the pair to assert the order of the 2 tokens.
pub static LB_PAIRS_INFO: Keymap<(String, String, u16), LbPairInformation, Json> =
    Keymap::new(b"lb_pairs_info");

// TODO: Figure out a storage type that avoids needing a separate PRESET_BIN_STEPS.
// In solidity this is: EnumerableMap.UintToUintMap
// I could make an empty struct that uses both of these internally, similar to the TreeUint24...
pub static PRESET_BIN_STEPS: Keyset<u16> = Keyset::new(b"preset_bin_steps");
pub static PRESETS: Keymap<u16, PairParameters> = Keymap::new(b"presets");
// TODO: Would a HashSet would be better for this? Store only addresses instead?
// This needs to be indexable while also being fixed cost to write, and elements must be unique...
// How are we ensuring unique elements?
pub static QUOTE_ASSET_WHITELIST: AppendStore<TokenType, Json> =
    AppendStore::new(b"quote_asset_whitelist");

// TODO: is this good?
// The Hashset<u16> will represent the "EnumerableSet.UintSet" from solidity.
// "The primary purpose of EnumerableSet.UintSet is to provide a convenient way to store, iterate, and retrieve
// elements in a set, while ensuring that they remain unique."

/// Mapping from a (tokenA, tokenB) to a set of available bin steps, this is used to keep track of the
/// bin steps that are already used for a pair.
/// The tokens are ordered to save gas, but they can be in the reverse order in the actual pair.
/// Always query one of the 2 tokens of the pair to assert the order of the 2 tokens.
pub static AVAILABLE_LB_PAIR_BIN_STEPS: Keymap<(String, String), HashSet<u16>, Json> =
    Keymap::new(b"available_lb_pair_bin_steps");

// TODO: decide on keeping this
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
    // pub fee_recipient: Addr,
    // pub lb_pair_implementation: Implementation,
    // pub lb_token_implementation: Implementation,
    // TODO: change to ContractInfo, or maybe get rid of these auth contracts...
    pub admin_auth: Contract,
    pub query_auth: Contract,
}

pub const EPHEMERAL_LB_PAIR: Item<EphemeralLbPair, Json> = Item::new(b"ephemeral_lb_pair");

#[cw_serde]
pub struct EphemeralLbPair {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub code_hash: String,
    pub created_by_owner: bool,
}
