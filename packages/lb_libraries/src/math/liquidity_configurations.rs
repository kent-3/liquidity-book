//! ### Liquidity Book Liquidity Configurations Library
//! Author: Kent and Haseeb
//!
//! This library contains functions to encode and decode the config of a pool and interact with the encoded Bytes32.

use super::{
    encoded::{Encoded, MASK_UINT24, MASK_UINT64},
    packed_u128_math::PackedUint128Math,
};
use crate::Bytes32;
use ethnum::U256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const OFFSET_ID: u8 = 0;
pub const OFFSET_DISTRIBUTION_Y: u8 = 24;
pub const OFFSET_DISTRIBUTION_X: u8 = 88;

pub const PRECISION: u64 = 1_000_000_000_000_000_000; // 1e18

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum LiquidityConfigurationsError {
    #[error("Liquidity Configurations Error: Distribution must be less than {PRECISION}")]
    InvalidConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, Copy, PartialEq, JsonSchema)]
pub struct LiquidityConfigurations(pub Bytes32);

impl LiquidityConfigurations {
    /**
     * @dev Encode the distributionX, distributionY and id into a single bytes32
     * @param distributionX The distribution of the first token
     * @param distributionY The distribution of the second token
     * @param id The id of the pool
     * @return config The encoded config as follows:
     * [0 - 24[: id
     * [24 - 88[: distributionY
     * [88 - 152[: distributionX
     * [152 - 256[: empty
     */
    pub fn encode_params(distribution_x: u64, distribution_y: u64, id: u32) -> Self {
        let mut config = Bytes32::default();

        config.set(distribution_x.into(), MASK_UINT64, OFFSET_DISTRIBUTION_X);
        config.set(distribution_y.into(), MASK_UINT64, OFFSET_DISTRIBUTION_Y);
        config.set(id.into(), MASK_UINT24, OFFSET_ID);

        Self(config)
    }

    /**
     * @dev Decode the distributionX, distributionY and id from a single bytes32
     * @param config The encoded config as follows:
     * [0 - 24[: id
     * [24 - 88[: distributionY
     * [88 - 152[: distributionX
     * [152 - 256[: empty
     * @return distributionX The distribution of the first token
     * @return distributionY The distribution of the second token
     * @return id The id of the bin to add the liquidity to
     */
    pub fn decode_params(config: Bytes32) -> (u64, u64, u32) {
        let distribution_x = config.decode_uint64(OFFSET_DISTRIBUTION_X);
        let distribution_y = config.decode_uint64(OFFSET_DISTRIBUTION_Y);
        let id = config.decode_uint24(OFFSET_ID);

        // TODO:
        // if (uint256(config) > type(uint152).max || distributionX > PRECISION || distributionY > PRECISION) {
        //     revert LiquidityConfigurations__InvalidConfig();
        // }

        (distribution_x, distribution_y, id)
    }

    /// Get the amounts and id from a config and amounts_in.
    ///
    /// # Arguments
    ///
    /// * `amounts_in` - The amounts to distribute as follows:
    ///     * [0 - 128[: x1
    ///     * [128 - 256[: x2
    ///
    /// # Returns
    ///
    /// * `amounts` - The distributed amounts as follows:
    ///     * [0 - 128[: x1
    ///     * [128 - 256[: x2
    /// * `id` - The id of the bin to add the liquidity to
    /// * `LiquidityConfigurationsError` - An error type for invalid config
    pub fn get_amounts_and_id(
        &self,
        amounts_in: Bytes32,
    ) -> Result<(Bytes32, u32), LiquidityConfigurationsError> {
        let (distribution_x, distribution_y, id) = Self::decode_params(self.0);

        let (x1, x2) = PackedUint128Math::decode(&amounts_in);

        // TODO: it could overflow during multiplication?

        // Cannot overflow as
        // max x1 or x2 = 2^128.
        // max distribution value= 10^18
        // PRECISION = 10^18

        // (2^128 * 10^18)/10^18 = 3.4 * 10^38 <  1.157 * 10^77

        let x1 = (U256::from(x1) * U256::from(distribution_x)) / U256::from(PRECISION);
        let x2 = (U256::from(x2) * U256::from(distribution_y)) / U256::from(PRECISION);

        let amounts = Bytes32::encode(x1.as_u128(), x2.as_u128());

        Ok((amounts, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethnum::U256;

    impl LiquidityConfigurations {
        pub fn new(
            distribution_x: u64,
            distribution_y: u64,
            id: u32,
        ) -> Result<Self, LiquidityConfigurationsError> {
            if (distribution_x > PRECISION) || (distribution_y > PRECISION) {
                Err(LiquidityConfigurationsError::InvalidConfig)
            } else {
                Ok(LiquidityConfigurations::encode_params(
                    distribution_x,
                    distribution_y,
                    id,
                ))
            }
        }

        pub fn update_distribution(
            &mut self,
            distribution_x: u64,
            distribution_y: u64,
        ) -> Result<(), LiquidityConfigurationsError> {
            if (distribution_x > PRECISION) || (distribution_y > PRECISION) {
                Err(LiquidityConfigurationsError::InvalidConfig)
            } else {
                self.0
                    .set(distribution_x.into(), MASK_UINT64, OFFSET_DISTRIBUTION_X);
                self.0
                    .set(distribution_y.into(), MASK_UINT64, OFFSET_DISTRIBUTION_Y);
                Ok(())
            }
        }
    }

    #[test]
    fn test_get_amounts_and_id_normal_case() {
        let distribution_x = (0.1 * PRECISION as f64) as u64;
        let distribution_y = (0.1 * PRECISION as f64) as u64;
        let id = 1;

        let lc = LiquidityConfigurations::encode_params(distribution_x, distribution_y, id);

        let amounts_in = Bytes32::encode(1000, 2000);

        let result = lc.get_amounts_and_id(amounts_in).unwrap();

        let (distribution_x, distribution_y, _id) = LiquidityConfigurations::decode_params(lc.0);

        let expected_x1_distributed =
            U256::from(1000u128) * U256::from(distribution_x) / U256::from(PRECISION);
        let expected_x2_distributed =
            U256::from(2000u128) * U256::from(distribution_y) / U256::from(PRECISION);

        let expected_amounts = Bytes32::encode(
            expected_x1_distributed.as_u128(),
            expected_x2_distributed.as_u128(),
        );

        assert_eq!(result, (expected_amounts, 1));
    }

    #[test]
    fn test_get_amounts_and_id_zero_case() {
        let lc = LiquidityConfigurations::default();
        let amounts_in = Bytes32::encode(0, 0);
        let result = lc.get_amounts_and_id(amounts_in).unwrap();

        let expected_amounts = Bytes32::encode(0, 0);

        assert_eq!(result, (expected_amounts, 0));
    }

    #[test]
    fn test_new_valid_config() {
        let lc = LiquidityConfigurations::new(500_000_000_000_000_000, 500_000_000_000_000_000, 1);
        assert!(lc.is_ok());
    }

    #[test]
    fn test_new_invalid_config_x() {
        let lc = LiquidityConfigurations::new(PRECISION + 1, 500_000_000_000_000_000, 1);
        assert_eq!(lc, Err(LiquidityConfigurationsError::InvalidConfig));
    }

    #[test]
    fn test_new_invalid_config_y() {
        let lc = LiquidityConfigurations::new(500_000_000_000_000_000, PRECISION + 1, 1);
        assert_eq!(lc, Err(LiquidityConfigurationsError::InvalidConfig));
    }

    #[test]
    fn test_update_distribution_valid() {
        let mut lc =
            LiquidityConfigurations::new(300_000_000_000_000_000, 300_000_000_000_000_000, 1)
                .unwrap();
        let result = lc.update_distribution(400_000_000_000_000_000, 400_000_000_000_000_000);
        assert!(result.is_ok());

        let (distribution_x, distribution_y, _id) = LiquidityConfigurations::decode_params(lc.0);
        assert_eq!(distribution_x, 400_000_000_000_000_000);
        assert_eq!(distribution_y, 400_000_000_000_000_000);
    }

    #[test]
    fn test_update_distribution_invalid_x() {
        let mut lc =
            LiquidityConfigurations::new(300_000_000_000_000_000, 300_000_000_000_000_000, 1)
                .unwrap();
        let result = lc.update_distribution(PRECISION + 1, 400_000_000_000_000_000);
        assert_eq!(result, Err(LiquidityConfigurationsError::InvalidConfig));
    }

    #[test]
    fn test_update_distribution_invalid_y() {
        let mut lc =
            LiquidityConfigurations::new(300_000_000_000_000_000, 300_000_000_000_000_000, 1)
                .unwrap();
        let result = lc.update_distribution(400_000_000_000_000_000, PRECISION + 1);
        assert_eq!(result, Err(LiquidityConfigurationsError::InvalidConfig));
    }
    #[test]
    fn test_equality() {
        let config1 = LiquidityConfigurations::new(100, 200, 1).unwrap();
        let config2 = LiquidityConfigurations::new(100, 200, 1).unwrap();
        assert_eq!(config1, config2);
    }

    #[test]
    fn test_debug_format() {
        let config = LiquidityConfigurations::new(100, 200, 1).unwrap();
        let debug_string = format!("{:?}", config);
        assert!(!debug_string.is_empty());
    }

    #[test]
    fn test_clone() {
        let config = LiquidityConfigurations::new(100, 200, 1).unwrap();
        let cloned_config = config.clone();
        assert_eq!(config, cloned_config);
    }
}
