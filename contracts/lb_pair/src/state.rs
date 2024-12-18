use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, ContractInfo, StdError, StdResult, Storage, Timestamp};
use lb_interfaces::lb_factory::ILbFactory;
use lb_interfaces::lb_pair::ContractStatus;
use lb_libraries::{Bytes32, OracleSample, PairParameters, TreeUint24};
// TODO: sort out viewing key strategy
use shade_protocol::swap::core::ViewingKey;
use shade_protocol::{
    secret_storage_plus::{Bincode2, Item, Map},
    swap::core::TokenType,
    Contract,
};

pub trait BinTree {
    fn add(&self, storage: &mut dyn Storage, id: u32) -> Result<bool, StdError>;
}

// TODO: exploring this idea
impl BinTree for Item<'_, TreeUint24, Bincode2> {
    fn add(&self, storage: &mut dyn Storage, id: u32) -> Result<bool, StdError> {
        let mut result: bool = true;

        BIN_TREE.update(storage, |mut tree| -> StdResult<_> {
            result = tree.add(id);
            Ok(tree)
        })?;

        Ok(result)
    }
}
pub const STATE: Item<State> = Item::new("state");
pub const CONTRACT_STATUS: Item<ContractStatus> = Item::new("contract_status");
pub const BIN_MAP: Map<u32, Bytes32> = Map::new("bins_map");
pub const BIN_TREE: Item<TreeUint24, Bincode2> = Item::new("bin_tree");
pub const ORACLE: Map<u16, OracleSample> = Map::new("oracle");
pub const EPHEMERAL_STORAGE: Item<EphemeralStruct> = Item::new("ephemeral_storage");

// TODO: use simplest possible storage types for things like parameters, reserves, protocol_fees.
// They can all be stored simply as bytes. No need to be serialized in any special way.
pub const FACTORY: Item<ILbFactory> = Item::new("factory");
pub const TOKEN_X: Item<TokenType> = Item::new("token_x");
pub const TOKEN_Y: Item<TokenType> = Item::new("token_y");
pub const BIN_STEP: Item<u16> = Item::new("bin_step");

pub const PARAMETERS: Item<PairParameters> = Item::new("pair_parameters");
pub const RESERVES: Item<Bytes32> = Item::new("reserves");
pub const PROTOCOL_FEES: Item<Bytes32> = Item::new("protocol_fees");

pub const LB_TOKEN: Item<ContractInfo> = Item::new("lb_token");

pub const HOOKS_PARAMETERS: Item<Bytes32> = Item::new("hooks_parameters");

// TODO: store some of these things under separate keys? especially reserves
#[cw_serde]
pub struct State {
    // Contract and creator information
    pub creator: Addr,
    pub factory: ContractInfo,
    // pub lb_token: ContractInfo,

    // Token and trading pair information
    // pub token_x: TokenType,
    // pub token_y: TokenType,
    pub bin_step: u16,
    // TODO: separate storage key for parameters
    // pub pair_parameters: PairParameters,
    pub viewing_key: ViewingKey,

    // Administrative and operational fields
    pub protocol_fees_recipient: Addr,
    pub admin_auth: Contract,
    pub last_swap_timestamp: Timestamp,
    // Financial fields
    // pub reserves: Bytes32,
    // pub protocol_fees: Bytes32,
}

#[cw_serde]
pub struct EphemeralStruct {
    pub lb_token_code_hash: String,
}
