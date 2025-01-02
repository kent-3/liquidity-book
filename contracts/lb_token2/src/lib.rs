//! # Liquidity Book Token
//!
//! The LBToken is an implementation of a multi-token.
//! It allows to create multi-SNIP20 represented by their ids.
//! Its implementation is really similar to the SNIP1155 standard the main difference
//! is that it doesn't do any call to the receiver contract to prevent reentrancy.
//! As it's only for SNIP20s, the uri function is not implemented.
//! The contract is made for batch operations.

mod contract;
mod execute;
mod query;
mod state;

pub use contract::{execute, instantiate, query};

// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_token2::LbTokenError as Error;

/// Alias for Result<T, LbRouterError>
pub type Result<T> = core::result::Result<T, Error>;
