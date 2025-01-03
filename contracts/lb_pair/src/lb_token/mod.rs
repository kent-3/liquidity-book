pub(crate) mod contract;
pub(crate) mod execute;
pub(crate) mod query;
pub(crate) mod state;

// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_token2::LbTokenError as Error;

/// Alias for Result<T, LbTokenError>
pub type Result<T, E = Error> = core::result::Result<T, E>;

use cosmwasm_std::{Deps, Env, Uint256};
use query::_is_approved_for_all;

/// Modifier to check if the spender is approved for all.
pub fn check_approval(deps: Deps, from: String, spender: String) -> Result<()> {
    if !_is_approved_for_all(deps, &from, &spender) {
        Err(Error::SpenderNotApproved { from, spender })
    } else {
        Ok(())
    }
}

// TODO: once again, what to do about address zero?
/// Modifier to check if the address is not zero or the contract itself.
pub fn not_address_zero_or_this(env: Env, account: String) -> Result<()> {
    if account == "" || account == env.contract.address.to_string() {
        Err(Error::AddressThisOrZero)
    } else {
        Ok(())
    }
}

/// Modifier to check if the length of the arrays are equal.
pub fn check_length(length_a: usize, length_b: usize) -> Result<()> {
    if length_a != length_b {
        Err(Error::InvalidLength)
    } else {
        Ok(())
    }
}
