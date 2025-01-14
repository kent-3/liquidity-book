//! ### Liquidity Book Error Library
//! Author: Kent and Haseeb
//!
//! This library reexports all of the different Error types for convenience.

pub use super::{
    bin_helper::BinError,
    fee_helper::FeeError,
    math::{
        liquidity_configurations::LiquidityConfigurationsError,
        packed_u128_math::PackedUint128MathError, u128x128_math::U128x128MathError,
        u256x256_math::U256x256MathError,
    },
    oracle_helper::OracleError,
    pair_parameter_helper::PairParametersError,
    price_helper::PriceError,
};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Generic(String),
    #[error(transparent)]
    BinError(#[from] BinError),
    #[error(transparent)]
    FeeError(#[from] FeeError),
    #[error(transparent)]
    LiquidityConfigurationsError(#[from] LiquidityConfigurationsError),
    #[error(transparent)]
    OracleError(#[from] OracleError),
    #[error(transparent)]
    PackedUint128MathError(#[from] PackedUint128MathError),
    #[error(transparent)]
    PairParametersError(#[from] PairParametersError),
    #[error(transparent)]
    PriceError(#[from] PriceError),
    #[error(transparent)]
    U128x128MathError(#[from] U128x128MathError),
    #[error(transparent)]
    U256x256MathError(#[from] U256x256MathError),
}
