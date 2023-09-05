//! Custom Errors for LB_TOKEN contract.

#![allow(unused)] // For beginning only.

use lb_libraries::bin_helper::BinError;
use lb_libraries::fee_helper::FeeError;
use lb_libraries::math::liquidity_configurations::LiquidityConfigurationsError;
use lb_libraries::math::u128x128_math::U128x128MathError;
use lb_libraries::math::u256x256_math::U256x256MathError;
use lb_libraries::oracle_helper::OracleError;
use lb_libraries::pair_parameter_helper::PairParametersError;

#[derive(thiserror::Error, Debug)]
pub enum LBTokenError {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    CwErr(#[from] cosmwasm_std::StdError),

    #[error("Invalid Error")]
    InvalidInput(String),

    #[error("Insufficient Funds")]
    InsufficientFunds,

    #[error("Insufficient Supply")]
    InsufficientSupply,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Spender Not Approved")]
    SpenderNotApproved,

    #[error("Self Approval")]
    SelfApproval,

    #[error("Already Approved")]
    AlreadyApproved,
}
