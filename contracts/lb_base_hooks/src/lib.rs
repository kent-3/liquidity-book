//! # Liquidity Book Base Hooks Contract
//!
//! Base contract for LBPair hooks.
//! This contract is meant to be inherited by any contract that wants to implement LBPair hooks.

pub mod contract;
pub mod execute;
pub mod query;
pub mod state;

// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_hooks::LbHooksError as Error;

/// Alias for Result<T, LbHooksError>
pub type Result<T, E = Error> = core::result::Result<T, E>;
