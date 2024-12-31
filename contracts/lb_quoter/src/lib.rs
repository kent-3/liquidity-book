//! # Liquidity Book Quoter
//!
//! Helper contract to determine best path through multiple markets.
//! This contract shouldn't be used on-chain as it consumes a lot of gas.
//! It should be used for off-chain purposes, like calculating the best path for a swap

mod contract;
mod helper;
mod query;
mod state;

pub use contract::{instantiate, query};

// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_quoter::LbQuoterError as Error;

/// Alias for Result<T, LbQuoterError>
pub type Result<T, E = Error> = core::result::Result<T, E>;
