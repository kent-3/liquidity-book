//! Helper Libraries

pub mod bin_helper;
pub mod constants;
pub mod error;
pub mod fee_helper;
pub mod hooks;
pub mod math;
pub mod oracle_helper;
pub mod pair_parameter_helper;
pub mod price_helper;

pub use self::{
    bin_helper::BinHelper,
    error::Error,
    fee_helper::FeeHelper,
    math::{
        encoded::Encoded, liquidity_configurations::LiquidityConfigurations,
        packed_u128_math::PackedUint128Math, sample_math::OracleSample, tree_math::TreeUint24,
        u128x128_math::U128x128Math, u256x256_math::U256x256Math,
    },
    oracle_helper::OracleMap,
    pair_parameter_helper::PairParameters,
    price_helper::PriceHelper,
};

pub type Bytes32 = [u8; 32];

// TODO: would there be any reason to do this?
//
// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};
//
// #[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Copy, JsonSchema, Eq, Hash)]
// pub struct Bytes32(pub [u8; 32]);
//
// impl From<[u8; 32]> for Bytes32 {
//     fn from(value: [u8; 32]) -> Self {
//         Bytes32(value)
//     }
// }
//
// impl AsRef<[u8]> for Bytes32 {
//     fn as_ref(&self) -> &[u8] {
//         &self.0
//     }
// }
//
// impl std::ops::Deref for Bytes32 {
//     type Target = [u8; 32];
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl std::ops::DerefMut for Bytes32 {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
