//! ### Liquidity Book Constants Library
//! Author: Kent and Haseeb
//!
//! Set of constants for Liquidity Book contracts.

use ethnum::{uint, U256};

pub static SCALE_OFFSET: u8 = 128;
pub static SCALE: U256 = U256::from_words(1, 0);

// TODO: should these all be U256 instead?

pub static PRECISION: u128 = 1_000_000_000_000_000_000; // 1e18
pub static SQUARED_PRECISION: u128 = PRECISION * PRECISION;

pub static MAX_FEE: u128 = 100_000_000_000_000_000; // 10% of 1e18
pub static MAX_PROTOCOL_SHARE: u16 = 2_500; // 25% of the fee

pub static BASIS_POINT_MAX: u16 = 10_000;

// (2^256 - 1) / (2 * log(2**128) / log(1.0001))
// 65251743116719673010965625540244653191619923014385985379600384103134737
// pub const MAX_LIQUIDITY_PER_BIN: U256 = U256::from_words(
//     191757638537527648490752896198554,       // high
//     268859549840696765395914820702069700113, // low
// );

// this is simpler and should work the same
pub const MAX_LIQUIDITY_PER_BIN: U256 =
    uint!("65251743116719673010965625540244653191619923014385985379600384103134737");

// joe-v2 used keccak256 instead of sha256
// sha256("LBPair.onFlashLoan")
/// The expected return after a successful flash loan
pub const CALLBACK_SUCCESS: [u8; 32] = [
    192u8, 152u8, 214u8, 165u8, 85u8, 164u8, 167u8, 247u8, 221u8, 39u8, 119u8, 142u8, 79u8, 52u8,
    100u8, 5u8, 98u8, 80u8, 209u8, 133u8, 255u8, 176u8, 24u8, 112u8, 223u8, 220u8, 225u8, 29u8,
    46u8, 110u8, 231u8, 7u8,
];
