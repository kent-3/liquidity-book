//! ### Custom Errors for LB_Pair contract.

use cosmwasm_std::{StdError, Uint128, Uint256};
// use lb_libraries::{
//     bin_helper::BinError,
//     fee_helper::FeeError,
//     math::{
//         liquidity_configurations::LiquidityConfigurationsError, u128x128_math::U128x128MathError,
//         u256x256_math::U256x256MathError,
//     },
//     oracle_helper::OracleError,
//     pair_parameter_helper::PairParametersError,
// };

#[derive(thiserror::Error, Debug)]
pub enum LbQuoterError {
    // Generic Errors
    #[error("Generic {0}")]
    Generic(String),

    // Error Wrappings from Dependencies
    #[error(transparent)]
    CwErr(#[from] StdError),
}
