use crate::{interfaces::lb_pair::ILbPair, libraries::hooks::HooksParameters, Bytes32};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    to_binary, Addr, Binary, ContractInfo, QuerierWrapper, StdError, StdResult, Uint256, WasmMsg,
};
use std::ops::Deref;

#[derive(thiserror::Error, Debug)]
pub enum LbHooksError {
    #[error("Invalid caller: {0}")]
    InvalidCaller(Addr),
    #[error("not linked")]
    NotLinked,
    #[error(transparent)]
    CwErr(#[from] StdError),
}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    OnHooksSet {
        hooks_parameters: HooksParameters,
        on_hooks_set_data: Option<Binary>,
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
    // pub lb_pair: Option<ContractInfo>,
    // TODO: let's see what happens when ILbPair gets serialized
    pub lb_pair: Option<ILbPair>,
}

#[cw_serde]
pub struct IsLinkedResponse {
    pub is_linked: bool,
}

/// A thin wrapper around `ContractInfo` that provides additional
/// methods to interact with an LB Hooks contract.
#[cw_serde]
pub struct ILbHooks(pub ContractInfo);

impl Deref for ILbHooks {
    type Target = ContractInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ILbHooks {
    pub fn on_hooks_set(
        &self,
        hooks_parameters: HooksParameters,
        on_hooks_set_data: Option<Binary>,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::OnHooksSet {
            hooks_parameters,
            on_hooks_set_data,
        };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }

    pub fn get_lb_pair(&self, querier: QuerierWrapper) -> StdResult<Addr> {
        querier
            .query_wasm_smart::<GetLbPairResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetLbPair {},
            )
            .and_then(|response| {
                response
                    .lb_pair
                    .ok_or(StdError::generic_err("No linked LB pair"))
            })
            .map(|ilb_pair| ilb_pair.address.clone())
    }
}
