//! # Liquidity Book Factory
//!
//! Contract used to deploy and register new LBPairs.
//! Enables setting fee parameters, flashloan fees and LBPair implementation.
//! Unless the `isOpen` is `true`, only the owner of the factory can create pairs.

mod contract;

pub use contract::{execute, instantiate, query, reply};

// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_factory::LbFactoryError as Error;

/// Alias for Result<T, LbFactoryError>
pub type Result<T, E = Error> = core::result::Result<T, E>;
