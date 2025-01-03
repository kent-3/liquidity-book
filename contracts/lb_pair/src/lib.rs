//! # Liquidity Book Pair
//!
//! The Liquidity Book Pair contract is the core contract of the Liquidity Book protocol

mod contract;
mod execute;
mod helper;
mod query;
mod state;

mod lb_token;

pub use contract::{execute, instantiate, query, reply};

// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_pair::LbPairError as Error;

/// Alias for Result<T, LbPairError>
pub type Result<T, E = Error> = core::result::Result<T, E>;
