use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, ContractInfo, Uint256};
use lb_libraries::Bytes32;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    OnHooksSet {
        hooks_parameters: Bytes32,
        on_hooks_set_data: Binary,
    },
    BeforeSwap {
        sender: String,
        to: String,
        swap_for_y: bool,
        amounts_in: Bytes32,
    },
    AfterSwap {
        sender: String,
        to: String,
        swap_for_y: bool,
        amounts_out: Bytes32,
    },
    BeforeFlashLoan {
        sender: String,
        to: String,
        amounts: Bytes32,
    },
    AfterFlashLoan {
        sender: String,
        to: String,
        fees: Bytes32,
        fees_received: Bytes32,
    },
    BeforeMint {
        sender: String,
        to: String,
        liquidity_configs: Vec<Bytes32>,
        amounts_received: Bytes32,
    },
    AfterMint {
        sender: String,
        to: String,
        liquidity_configs: Vec<Bytes32>,
        amounts_in: Bytes32,
    },
    BeforeBurn {
        sender: String,
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts_to_burn: Vec<Uint256>,
    },
    AfterBurn {
        sender: String,
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts_to_burn: Vec<Uint256>,
    },
    BeforeBatchTransferFrom {
        sender: String,
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    },
    AfterBatchTransferFrom {
        sender: String,
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetLbPairResponse)]
    GetLbPair,
    #[returns(IsLinkedResponse)]
    IsLinked,
}

#[cw_serde]
pub struct GetLbPairResponse {
    pub lb_pair: Option<ContractInfo>,
}

#[cw_serde]
pub struct IsLinkedResponse {
    pub is_linked: bool,
}
