//! ### Custom Errors for LB_Hooks contract.

use cosmwasm_std::{Addr, StdError};

#[derive(thiserror::Error, Debug)]
pub enum LbHooksError {
    #[error("Invalid caller: {0}")]
    InvalidCaller(Addr),
    #[error("not linked")]
    NotLinked,
    #[error(transparent)]
    CwErr(#[from] StdError),
}
