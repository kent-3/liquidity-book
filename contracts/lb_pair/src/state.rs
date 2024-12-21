use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, ContractInfo, StdError, StdResult, Storage};
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

// TODO: exploring this idea

pub trait BinTree {
    fn add(&self, storage: &mut dyn Storage, id: u32) -> Result<bool, StdError>;
}

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

pub const FACTORY: Item<ILbFactory> = Item::new("factory");
pub const LB_TOKEN: Item<ContractInfo> = Item::new("lb_token");

pub const TOKEN_X: Item<TokenType> = Item::new("token_x");
pub const TOKEN_Y: Item<TokenType> = Item::new("token_y");
pub const BIN_STEP: Item<u16> = Item::new("bin_step");

pub const BIN_MAP: Map<u32, Bytes32> = Map::new("bin_map");
pub const BIN_TREE: Item<TreeUint24, Bincode2> = Item::new("bin_tree");
pub const ORACLE: Map<u16, OracleSample> = Map::new("oracle");

pub const PARAMETERS: Item<PairParameters> = Item::new("pair_parameters");
pub const RESERVES: Item<Bytes32> = Item::new("reserves");
pub const PROTOCOL_FEES: Item<Bytes32> = Item::new("protocol_fees");
pub const HOOKS_PARAMETERS: Item<Bytes32> = Item::new("hooks_parameters");

pub const EPHEMERAL_LB_TOKEN: Item<EphemeralLbToken> = Item::new("ephemeral_lb_token");
pub const EPHEMERAL_FLASH_LOAN: Item<EphemeralFlashLoan> = Item::new("ephemeral_flash_loan");

// TODO: clean this up
#[cw_serde]
pub struct State {
    // Contract and creator information
    pub creator: Addr,
    // pub factory: ContractInfo,
    // pub lb_token: ContractInfo,

    // Token and trading pair information
    // pub token_x: TokenType,
    // pub token_y: TokenType,
    // pub pair_parameters: PairParameters,
    pub viewing_key: ViewingKey,

    // Administrative and operational fields
    pub protocol_fees_recipient: Addr,
    pub admin_auth: Contract,
    // why did we need this?
    // pub last_swap_timestamp: Timestamp,

    // Financial fields
    // pub reserves: Bytes32,
    // pub protocol_fees: Bytes32,
}

#[cw_serde]
pub struct EphemeralLbToken {
    pub code_hash: String,
}

#[cw_serde]
pub struct EphemeralFlashLoan {
    pub reserves_before: Bytes32,
    pub total_fees: Bytes32,
    pub sender: Addr,
    pub receiver: Addr,
    pub amounts: Bytes32,
}

// TODO: (maybe) We could just have one storage key that can store any of the epehemeral storage
// types, as long as they are serializable. But maybe it's better not to?
// example:
//     EPHEMERAL_STORAGE.save(
//     deps.storage,
//     &to_binary(&EphemeralLbToken {
//         code_hash: msg.lb_token_implementation.code_hash,
//     })?,
// )?;

pub const EPHEMERAL_STORAGE: Item<Binary> = Item::new("ephemeral_storage");
