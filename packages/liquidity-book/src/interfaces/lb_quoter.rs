use super::lb_router::Version;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{ContractInfo, Uint128};
// TODO: copy these instead of using shade_protocol as a dependency
use shade_protocol::{
    swap::core::TokenType,
    utils::{asset::RawContract, Query},
};

use crate::libraries::math::{u128x128_math::U128x128MathError, u256x256_math::U256x256MathError};
use cosmwasm_std::StdError;

#[derive(thiserror::Error, Debug)]
pub enum LbQuoterError {
    #[error("InvalidLength")]
    InvalidLength,

    // Error Wrappings from Dependencies
    #[error(transparent)]
    CwErr(#[from] StdError),
    #[error(transparent)]
    U128Err(#[from] U128x128MathError),
    #[error(transparent)]
    U256Err(#[from] U256x256MathError),
}

#[cw_serde]
#[derive(Default)]
pub struct Quote {
    pub route: Vec<TokenType>,
    pub pairs: Vec<ContractInfo>,
    pub bin_steps: Vec<u16>,
    pub versions: Vec<Version>,
    pub amounts: Vec<Uint128>,
    pub virtual_amounts_without_slippage: Vec<Uint128>,
    pub fees: Vec<Uint128>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub factory_v2_2: Option<RawContract>,
    pub router_v2_2: Option<RawContract>,
}

#[cw_serde]
pub struct ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FactoryV2_2Response)]
    GetFactoryV2_2,
    #[returns(RouterV2_2Response)]
    GetRouterV2_2,
    #[returns(Quote)]
    FindBestPathFromAmountIn {
        route: Vec<TokenType>,
        amount_in: Uint128,
    },
    #[returns(Quote)]
    FindBestPathFromAmountOut {
        route: Vec<TokenType>,
        amount_out: Uint128,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct FactoryV2_2Response {
    pub factory_v2_2: Option<ContractInfo>,
}

#[cw_serde]
pub struct RouterV2_2Response {
    pub router_v2_2: Option<ContractInfo>,
}
