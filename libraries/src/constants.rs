//! ### Liquidity Book Constants Library
//! Author: Kent
//!
//! Set of constants for Liquidity Book contracts.

use ethnum::U256;
// use cosmwasm_std::Uint256;

pub static SCALE_OFFSET: u8 = 128;

// needs to be this number (128 trailing zeroes):
// 0b1_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000

// use this one for cosmwasm Uint256:
// pub static SCALE: Uint256 = Uint256::from_le_bytes([
//     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
// ]);

// use this one for ethnum U256:
pub static SCALE: U256 = U256::from_words(1, 0);

pub static PRECISION: u128 = 1_000_000_000_000_000_000; // 1e18 coz it's ethereum
pub static SQUARED_PRECISION: u128 = PRECISION * PRECISION;

pub static MAX_FEE: u128 = 100_000_000_000_000_000; // 10%
pub static MAX_PROTOCOL_SHARE: u32 = 2_500; // 25% of the fee

pub static BASIS_POINT_MAX: u32 = 10_000;
