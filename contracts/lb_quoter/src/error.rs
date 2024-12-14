//! ### Custom Errors for LB_Quoter contract.

use cosmwasm_std::StdError;
use lb_libraries::math::{u128x128_math::U128x128MathError, u256x256_math::U256x256MathError};

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
