//! ### Custom Errors for LB_Hooks contract.

use cosmwasm_std::StdError;

#[derive(thiserror::Error, Debug)]
pub enum LbHooksError {
    #[error(transparent)]
    CwErr(#[from] StdError),
}
