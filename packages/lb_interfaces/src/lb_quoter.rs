use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ContractInfo, Uint128};
// TODO: copy these instead of using shade_protocol as a dependency
use shade_protocol::utils::{asset::RawContract, InstantiateCallback, Query};

#[cw_serde]
pub struct Quote {
    pub route: Vec<String>,
    pub pairs: Vec<String>,
    pub bin_steps: Vec<u16>,
    pub versions: Vec<String>,
    pub amounts: Vec<Uint128>,
    pub virtual_amounts_without_slippage: Vec<Uint128>,
    pub fees: Vec<Uint128>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub factory_v1: Option<RawContract>,
    pub router_v1: Option<RawContract>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FactoryV1Response)]
    GetFactoryV1,
    #[returns(RouterV1Response)]
    GetRouterV1,
    #[returns(QuoteResponse)]
    FindBestPathFromAmountIn {
        route: Vec<ContractInfo>,
        amount_in: Uint128,
    },
    #[returns(QuoteResponse)]
    FindBestPathFromAmountOut {
        route: Vec<ContractInfo>,
        amount_out: Uint128,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct FactoryV1Response {
    pub factory_v1: Option<ContractInfo>,
}

#[cw_serde]
pub struct RouterV1Response {
    pub router_v1: Option<ContractInfo>,
}

#[cw_serde]
pub struct QuoteResponse {
    pub quote: Quote,
}
