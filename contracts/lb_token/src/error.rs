//! Custom Errors for LB_TOKEN contract.

#![allow(unused)] // For beginning only.

use lb_libraries::{
    bin_helper::BinError,
    fee_helper::FeeError,
    math::{
        liquidity_configurations::LiquidityConfigurationsError,
        u128x128_math::U128x128MathError,
        u256x256_math::U256x256MathError,
    },
    oracle_helper::OracleError,
    pair_parameter_helper::PairParametersError,
};

// TODO - Rework this for SNIP1155 errors (or not?)
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
