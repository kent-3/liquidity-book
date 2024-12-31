//! # Liquidity Book Router
//!
//! Main contract to interact with to swap and manage liquidity on Amber exchange.

mod contract;
mod execute;
mod query;
mod state;

pub use contract::{execute, instantiate, query, reply};

// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_router::LbRouterError as Error;

/// Alias for Result<T, LbRouterError>
pub type Result<T> = core::result::Result<T, Error>;
