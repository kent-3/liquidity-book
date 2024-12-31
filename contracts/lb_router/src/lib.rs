//! # Liquidity Book Router
//!
//! Main contract to interact with to swap and manage liquidity on Amber exchange.

#![warn(missing_docs)]

mod contract;
mod execute;
mod query;
mod state;

// TODO: Error types should be defined in the interfaces.
mod error;
mod prelude;

pub use contract::{execute, instantiate, query, reply};
