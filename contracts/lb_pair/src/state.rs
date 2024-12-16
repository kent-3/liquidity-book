use lb_interfaces::lb_pair::ContractStatus;
use lb_libraries::{
    math::{sample_math::OracleSample, tree_math::TreeUint24},
    pair_parameter_helper::PairParameters,
    types::Bytes32,
};
// TODO: sort out viewing key strategy
use shade_protocol::swap::core::ViewingKey;
use shade_protocol::{
    c_std::{Addr, ContractInfo, Timestamp},
    cosmwasm_schema::cw_serde,
    secret_storage_plus::{Bincode2, Item, Map},
    swap::core::TokenType,
    Contract,
};

pub const STATE: Item<State> = Item::new("state");
pub const CONTRACT_STATUS: Item<ContractStatus> = Item::new("contract_status");
pub const BIN_MAP: Map<u32, Bytes32> = Map::new("bins_map");
pub const BIN_TREE: Item<TreeUint24, Bincode2> = Item::new("bin_tree");
pub const ORACLE: Map<u16, OracleSample> = Map::new("oracle");
pub const EPHEMERAL_STORAGE: Item<EphemeralStruct> = Item::new("ephemeral_storage");

// TODO: use simplest possible storage types for things like parameters, reserves, protocol_fees.
// They can all be stored simply as bytes. No need to be serialized in any special way.
pub const LB_TOKEN: Item<ContractInfo> = Item::new("lb_token");
pub const PARAMETERS: Item<Bytes32> = Item::new("pair_parameters");
pub const RESERVES: Item<Bytes32> = Item::new("reserves");
pub const PROTOCOL_FEES: Item<Bytes32> = Item::new("protocol_fees");
pub const HOOKS_PARAMETERS: Item<Bytes32> = Item::new("hooks_parameters");

// TODO: store some of these things under separate keys? especially reserves
#[cw_serde]
pub struct State {
    // Contract and creator information
    pub creator: Addr,
    pub factory: ContractInfo,
    pub lb_token: ContractInfo,

    // Token and trading pair information
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    // TODO: separate storage key for parameters
    pub pair_parameters: PairParameters,
    pub viewing_key: ViewingKey,

    // Administrative and operational fields
    pub protocol_fees_recipient: Addr,
    pub admin_auth: Contract,
    pub last_swap_timestamp: Timestamp,

    // Financial fields
    pub reserves: Bytes32,
    pub protocol_fees: Bytes32,
}

#[cw_serde]
pub struct EphemeralStruct {
    pub lb_token_code_hash: String,
}
