//! # Liquidity Book Router
//!
//! Main contract to interact with to swap and manage liquidity on Amber exchange.

#![warn(missing_docs)]

mod prelude;

mod contract;
mod execute;
mod query;
mod state;

pub use contract::{execute, instantiate, query, reply};
