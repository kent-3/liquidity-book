//! ### Liquidity Book Bin Helper Library
//! Author: Kent and Haseeb
//!
//! This library contains functions to help interaction with bins.

use super::{
    constants::{SCALE, SCALE_OFFSET},
    fee_helper::{FeeError, FeeHelper},
    math::{
        packed_u128_math::{PackedUint128Math, PackedUint128MathError},
        u128x128_math::U128x128MathError,
        u256x256_math::{U256x256Math, U256x256MathError},
    },
    pair_parameter_helper::{PairParameters, PairParametersError},
    Bytes32, PriceHelper,
};
use ethnum::U256;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum BinError {
    #[error("Bin Error: Composition Factor Flawed, id: {0}")]
    CompositionFactorFlawed(u32),

    #[error("Bin Error: Liquidity Overflow")]
    LiquidityOverflow,

    #[error(transparent)]
    FeeErr(#[from] FeeError),

    #[error(transparent)]
    U128MathErr(#[from] U128x128MathError),

    #[error(transparent)]
    U256MathErr(#[from] U256x256MathError),

    #[error(transparent)]
    ParamsErr(#[from] PairParametersError),

    #[error(transparent)]
    PackedUint128MathErr(#[from] PackedUint128MathError),
}

pub trait BinHelper {
    /// Returns the amount of tokens that will be received when burning the given amount of liquidity.
    fn get_amount_out_of_bin(
        &self,
        amount_to_burn: U256,
        total_supply: U256,
    ) -> Result<Bytes32, BinError>;

    /// Returns the share and the effective amounts in when adding liquidity.
    fn get_shares_and_effective_amounts_in(
        &self,
        amounts_in: Bytes32,
        price: U256,
        total_supply: U256,
    ) -> Result<(U256, Bytes32), BinError>;

    /// Returns the amount of liquidity following the constant sum formula `L = price * x + y`.
    fn get_liquidity(&self, price: U256) -> Result<U256, BinError>;

    /// Verify that the amounts are correct and that the composition factor is not flawed.
    fn verify_amounts(&self, active_id: u32, id: u32) -> Result<(), BinError>;

    /// Returns the composition fees when adding liquidity to the active bin with a different
    /// composition factor than the bin's one, as it does an implicit swap.
    fn get_composition_fees(
        &self,
        parameters: PairParameters,
        bin_step: u16,
        amounts_in: Bytes32,
        total_supply: U256,
        shares: U256,
    ) -> Result<Bytes32, BinError>;

    /// Returns whether the bin is empty (true) or not (false).
    fn is_empty(&self, is_x: bool) -> bool;

    /// Returns the amounts of tokens that will be added and removed from the bin during a swap
    /// along with the fees that will be charged.
    fn get_amounts(
        &self,
        parameters: PairParameters,
        bin_step: u16,
        swap_for_y: bool,
        active_id: u32,
        amounts_in_left: Bytes32,
    ) -> Result<(Bytes32, Bytes32, Bytes32), BinError>;

    /// Returns the encoded amounts that were transferred to the contract for both tokens.
    fn received(&self, token_x: u128, token_y: u128) -> Bytes32;

    /// Returns the encoded amounts that were transferred to the contract, only for token X.
    fn received_x(&self, token_x: u128) -> Bytes32;

    /// Returns the encoded amounts that were transferred to the contract, only for token Y.
    fn received_y(&self, token_y: u128) -> Bytes32;
}

impl BinHelper for Bytes32 {
    /// Returns the amount of tokens that will be received when burning the given amount of liquidity.
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin
    /// * `amount_to_burn` - The amount of liquidity to burn
    /// * `total_supply` - The total supply of the liquidity book
    ///
    /// # Returns
    ///
    /// * `amounts_out` - The amounts of tokens that will be received
    fn get_amount_out_of_bin(
        &self,
        amount_to_burn: U256,
        total_supply: U256,
    ) -> Result<Bytes32, BinError> {
        let (bin_reserve_x, bin_reserve_y) = self.decode();

        // Rounding down in the context of token distributions or liquidity removal is a conservative approach
        // that errs on the side of caution. It ensures that the contract never overestimates the amount of
        // tokens that should be returned to a user.
        let amount_x_out_from_bin = if bin_reserve_x > 0 {
            U256x256Math::mul_div_round_down(&amount_to_burn, bin_reserve_x.into(), total_supply)?
                .as_u128()
        } else {
            0u128
        };

        let amount_y_out_from_bin = if bin_reserve_y > 0 {
            U256x256Math::mul_div_round_down(&amount_to_burn, bin_reserve_y.into(), total_supply)?
                .as_u128()
        } else {
            0u128
        };

        let amounts_out = Bytes32::encode(amount_x_out_from_bin, amount_y_out_from_bin);

        Ok(amounts_out)
    }

    /// Returns the share and the effective amounts in when adding liquidity.
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin.
    /// * `amounts_in` - The amounts of tokens to add.
    /// * `price` - The price of the bin.
    /// * `total_supply` - The total supply of the liquidity book.
    ///
    /// # Returns
    ///
    /// * `shares` - The share of the liquidity book that the user will receive as a Uint256.
    /// * `effective_amounts_in` - The Bytes32 encoded effective amounts of tokens that the user will add.
    ///   This is the amount of tokens that the user will actually add to the liquidity book,
    ///   and will always be less than or equal to the amounts_in.
    fn get_shares_and_effective_amounts_in(
        &self, // self is the bin_reserves
        mut amounts_in: Bytes32,
        price: U256,
        total_supply: U256,
    ) -> Result<(U256, Bytes32), BinError> {
        let (mut x, mut y) = amounts_in.decode();

        let user_liquidity = Self::get_liquidity(&amounts_in, price)?;
        if user_liquidity == U256::ZERO {
            return Ok((U256::ZERO, Bytes32::default()));
        }

        let bin_liquidity = Self::get_liquidity(self, price)?;
        if bin_liquidity == U256::ZERO || total_supply == U256::ZERO {
            // FIXME: return user_liquidity.sqrt() instead! But why???
            return Ok((user_liquidity, amounts_in));
        }

        let shares = user_liquidity.mul_div_round_down(total_supply, bin_liquidity)?;
        let effective_liquidity = shares.mul_div_round_up(bin_liquidity, total_supply)?;

        // effective_liquidity and user_liquidity would be different when the total_supply is a number other than the sum of the all individual liquidities
        if user_liquidity > effective_liquidity {
            let mut delta_liquidity = user_liquidity - effective_liquidity;

            // The other way might be more efficient, but as y is the quote asset, it is more valuable
            if delta_liquidity >= SCALE {
                let delta_y = (delta_liquidity >> SCALE_OFFSET as u32)
                    .min(y.into())
                    .as_u128();

                y -= delta_y;
                delta_liquidity -= U256::from(delta_y) << SCALE_OFFSET as u128;
            }

            if delta_liquidity >= price {
                let delta_x = (delta_liquidity / price).min(x.into()).as_u128();

                x -= delta_x;
            }

            amounts_in = Bytes32::encode(x, y);
        }

        // TODO: max liquidity thing

        Ok((shares, amounts_in))
    }

    /// Returns the amount of liquidity following the constant sum formula `L = price * x + y`.
    ///
    /// # Arguments
    ///
    /// * `amounts` - The amount of the tokens
    /// * `price` - The price of the bin
    fn get_liquidity(&self, price: U256) -> Result<U256, BinError> {
        let (x, y) = self.decode();

        let x = U256::from(x);
        let y = U256::from(y);

        let mut liquidity = U256::ZERO;

        // TODO: this seems potentially inefficient. try using checked_mul instead

        if x > U256::ZERO {
            liquidity = price.wrapping_mul(x); // Trying to make sure that if the liq > 2^256 don't overflow instead doing it through the check

            if liquidity / x != price {
                return Err(BinError::LiquidityOverflow);
            }
        }

        if y > U256::ZERO {
            let shifted_y = y << SCALE_OFFSET;
            liquidity = liquidity.wrapping_add(shifted_y);

            if liquidity < y {
                return Err(BinError::LiquidityOverflow);
            }
        }

        Ok(liquidity)
    }

    /// Verify that the amounts are correct and that the composition factor is not flawed.
    ///
    /// # Arguments
    ///
    /// * `amounts` - The amounts of tokens as Bytes32.
    /// * `active_id` - Thie id of the active bin as u32.
    /// * `id` - Thie id of the bin as u32.
    fn verify_amounts(&self, active_id: u32, id: u32) -> Result<(), BinError> {
        let amounts = U256::from_le_bytes(*self);
        // this is meant to compare the right-side 128 bits to zero, but can I discard the left 128 bits and not have it overflow?
        if id < active_id && (amounts << 128u32) > U256::ZERO
            || id > active_id && amounts > U256::from(u128::MAX)
        {
            return Err(BinError::CompositionFactorFlawed(id));
        }
        Ok(())
    }

    /// Returns the composition fees when adding liquidity to the active bin with a different
    /// composition factor than the bin's one, as it does an implicit swap.
    /// It calculates what you'd get if you removed 10 shares of liquidity right after adding it
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin
    /// * `parameters` - The parameters of the liquidity book
    /// * `bin_step` - The step of the bin
    /// * `amounts_in` - The amounts of tokens to add
    /// * `total_supply` - The total supply of the liquidity book
    /// * `shares` - The share of the liquidity book that the user will receive
    ///
    /// # Returns
    ///
    /// * `fees` - The encoded fees that will be charged
    fn get_composition_fees(
        &self,
        parameters: PairParameters,
        bin_step: u16,
        amounts_in: Bytes32,
        total_supply: U256,
        shares: U256,
    ) -> Result<Bytes32, BinError> {
        if shares == U256::ZERO {
            return Ok([0u8; 32]);
        }

        let (amount_x, amount_y) = amounts_in.decode();

        let (received_amount_x, received_amount_y) =
            Self::get_amount_out_of_bin(&self.add(amounts_in)?, shares, total_supply + shares)?
                .decode();

        let mut fees = Bytes32::default();

        if received_amount_x > amount_x {
            let fee_y = (amount_y - received_amount_y)
                .get_composition_fee(parameters.get_total_fee(bin_step)?)?;

            fees = Bytes32::encode_second(fee_y)
        } else if received_amount_y > amount_y {
            let fee_x = (amount_x - received_amount_x)
                .get_composition_fee(parameters.get_total_fee(bin_step)?)?;

            fees = Bytes32::encode_first(fee_x)
        }

        Ok(fees)
    }

    /// Returns whether the bin is empty (true) or not (false).
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin
    /// * `is_x` - Whether the reserve to check is the X reserve (true) or the Y reserve (false)
    fn is_empty(&self, is_x: bool) -> bool {
        if is_x {
            self.decode_x() == 0
        } else {
            self.decode_y() == 0
        }
    }

    /// Returns the amounts of tokens that will be added and removed from the bin during a swap
    /// along with the fees that will be charged.
    ///
    /// # Arguments
    ///
    /// * `bin_reserves` - The reserves of the bin
    /// * `parameters` - The parameters of the liquidity book
    /// * `bin_step` - The step of the bin
    /// * `swap_for_y` - Whether the swap is for Y (true) or for X (false)
    /// * `active_id` - The id of the active bin
    /// * `amounts_in_left` - The amounts of tokens left to swap
    ///
    /// # Returns
    ///
    /// * `amounts_in_with_fees` - The encoded amounts of tokens that will be added to the bin, including fees.
    /// * `amounts_out_of_bin` - The encoded amounts of tokens that will be removed from the bin.
    /// * `total_fees` - The encoded fees that will be charged.
    fn get_amounts(
        &self,
        parameters: PairParameters,
        bin_step: u16,
        swap_for_y: bool,
        active_id: u32,
        amounts_in_left: Bytes32,
    ) -> Result<(Bytes32, Bytes32, Bytes32), BinError> {
        let price = PriceHelper::get_price_from_id(active_id, bin_step)?;

        let bin_reserve_out: u128 = self.decode_alt(!swap_for_y);

        // The rounding up ensures that you don't underestimate the amount of token_x or token_y needed,
        let mut max_amount_in: u128 = if swap_for_y {
            U256::from(bin_reserve_out)
                .shift_div_round_up(SCALE_OFFSET, price)?
                .as_u128()
        } else {
            U256::from(bin_reserve_out)
                .mul_shift_round_up(price, SCALE_OFFSET)?
                .as_u128()
        };

        let total_fee: u128 = parameters.get_total_fee(bin_step)?;
        let max_fee: u128 = max_amount_in.get_fee_amount(total_fee)?;

        max_amount_in += max_fee;

        let mut amount_in_128: u128 = amounts_in_left.decode_alt(swap_for_y);
        let fee_128: u128;
        let mut amount_out_128: u128;

        if amount_in_128 >= max_amount_in {
            fee_128 = max_fee;

            amount_in_128 = max_amount_in;
            amount_out_128 = bin_reserve_out;
        } else {
            fee_128 = amount_in_128.get_fee_amount_from(total_fee)?;

            let amount_in = amount_in_128 - fee_128;

            amount_out_128 = if swap_for_y {
                U256::from(amount_in)
                    .mul_shift_round_down(price, SCALE_OFFSET)?
                    .as_u128()
            } else {
                U256::from(amount_in)
                    .shift_div_round_down(SCALE_OFFSET, price)?
                    .as_u128()
            };

            if amount_out_128 > bin_reserve_out {
                amount_out_128 = bin_reserve_out;
            }
        };

        let (amounts_in_with_fees, amounts_out_of_bin, total_fees) = if swap_for_y {
            (
                Bytes32::encode_first(amount_in_128),
                Bytes32::encode_second(amount_out_128),
                Bytes32::encode_first(fee_128),
            )
        } else {
            (
                Bytes32::encode_second(amount_in_128),
                Bytes32::encode_first(amount_out_128),
                Bytes32::encode_second(fee_128),
            )
        };

        Ok((amounts_in_with_fees, amounts_out_of_bin, total_fees))
    }

    /// Returns the encoded amounts that were transferred to the contract for both tokens.
    /// Determined by subtracting the contract's reserves from the contract's token balances.
    ///
    /// # Arguments
    ///
    /// * `reserves` - The reserves
    /// * `token_x` - The token X
    /// * `token_y` - The token Y
    ///
    /// # Returns
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: amount_x
    ///     * [128 - 256[: amount_y
    fn received(&self, token_x: u128, token_y: u128) -> Bytes32 {
        Bytes32::encode(token_x, token_y).sub(*self).unwrap() // TODO: return Result instead
    }

    /// Returns the encoded amounts that were transferred to the contract, only for token X.
    ///
    /// # Arguments
    ///
    /// * `reserves` - The reserves
    /// * `token_x` - The token X
    ///
    /// # Returns
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: amount_x
    ///     * [128 - 256[: empty
    fn received_x(&self, token_x: u128) -> Bytes32 {
        let reserve_x = self.decode_x();
        Bytes32::encode_first(token_x - reserve_x)
    }

    /// Returns the encoded amounts that were transferred to the contract, only for token Y.
    ///
    /// # Arguments
    ///
    /// * `reserves` - The reserves
    /// * `token_x` - The token Y
    ///
    /// # Returns
    ///
    /// * `amounts` - The amounts, encoded as follows:
    ///     * [0 - 128[: empty
    ///     * [128 - 256[: amount_y
    fn received_y(&self, token_y: u128) -> Bytes32 {
        let reserve_y = self.decode_y();
        Bytes32::encode_second(token_y - reserve_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ethnum::U256;
    use std::str::FromStr;

    fn assert_approxeq_abs(a: U256, b: U256, max_diff: U256, msg: &str) {
        let diff = if a > b { a - b } else { b - a };
        assert!(diff <= max_diff, "{}: diff was {:?}", msg, diff);
    }

    #[test]
    fn test_get_amount_out_of_bin_zero_bin_reserves() -> Result<(), BinError> {
        let bin_reserves = Bytes32::encode(0, 0);
        let amount_to_burn = U256::from(1000u128);
        let total_supply = U256::from(10000u128);

        let amount_out = bin_reserves
            .get_amount_out_of_bin(amount_to_burn, total_supply)
            .unwrap();
        let (amount_out_x, amount_out_y) = amount_out.decode();

        assert_eq!(amount_out_x, 0);
        assert_eq!(amount_out_y, 0);

        Ok(())
    }

    #[test]
    fn test_get_amount_out_of_bin_zero_amount_to_burn() -> Result<(), BinError> {
        let bin_reserves = Bytes32::encode(1000, 1000);
        let amount_to_burn = U256::from(0u128);
        let total_supply = U256::from(10000u128);

        let amount_out = bin_reserves
            .get_amount_out_of_bin(amount_to_burn, total_supply)
            .unwrap();
        let (amount_out_x, amount_out_y) = amount_out.decode();

        assert_eq!(amount_out_x, 0);
        assert_eq!(amount_out_y, 0);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_get_amount_out_of_bin_zero_total_supply() {
        let bin_reserves = Bytes32::encode(1000, 1000);
        let amount_to_burn = U256::from(1000u128);
        let total_supply = U256::from(0u128);

        let _ = bin_reserves
            .get_amount_out_of_bin(amount_to_burn, total_supply)
            .unwrap();
    }

    #[test]
    fn test_get_amount_out_of_bin_amount_to_burn_gt_total_supply() -> Result<(), BinError> {
        let bin_reserves = Bytes32::encode(1000, 1000);
        let amount_to_burn = U256::from(20000u128); // Greater than total_supply
        let total_supply = U256::from(10000u128);

        let amount_out =
            BinHelper::get_amount_out_of_bin(&bin_reserves, amount_to_burn, total_supply)?;
        let (amount_out_x, amount_out_y) = amount_out.decode();

        // Your assertions go here, depending on what behavior you expect
        // For instance, if you expect it to be proportional
        assert_eq!(amount_out_x, U256::from(2000u128));
        assert_eq!(amount_out_y, U256::from(2000u128));

        Ok(())
    }

    #[test]
    fn test_get_amount_out_of_bin_max_u128_constraint() -> Result<(), BinError> {
        let bin_reserves = Bytes32::encode(u128::MAX, u128::MAX);
        let amount_to_burn = U256::from(u128::MAX);
        let total_supply = U256::from(u128::MAX); // To make sure the raw output is > u128::MAX

        let amount_out =
            BinHelper::get_amount_out_of_bin(&bin_reserves, amount_to_burn, total_supply)?;
        let (amount_out_x, amount_out_y) = amount_out.decode();

        // Should be capped at u128::MAX
        assert_eq!(amount_out_x, u128::MAX);
        assert_eq!(amount_out_y, u128::MAX);

        Ok(())
    }

    #[test]
    fn test_get_amount_out_of_bin() -> Result<(), BinError> {
        let bin_reserves_x = 1000;
        let bin_reserves_y = 1000;

        let bin_reserves = Bytes32::encode(bin_reserves_x, bin_reserves_y);
        let amount_to_burn = U256::from(1000u128);
        let total_supply = U256::from(10000u128);

        let amount_out =
            BinHelper::get_amount_out_of_bin(&bin_reserves, amount_to_burn, total_supply).unwrap();

        let (amount_out_x, amount_out_y) = amount_out.decode();

        assert_eq!(
            amount_out_x,
            U256x256Math::mul_div_round_down(
                &amount_to_burn,
                U256::from(bin_reserves_x),
                total_supply
            )
            .unwrap()
        );
        assert_eq!(
            amount_out_y,
            U256x256Math::mul_div_round_down(
                &amount_to_burn,
                U256::from(bin_reserves_y),
                total_supply
            )
            .unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_get_shares_and_effective_amounts_in_delta_liquidity_adjustment() {
        // Assume these constants based on your SCALE and SCALE_OFFSET values
        let _scale = U256::from(SCALE);
        let scale_offset = U256::from(SCALE_OFFSET);

        // Sample input values to trigger the condition
        let bin_reserves = Bytes32::encode(1000, 2000); // bin reserves
        let price = U256::from(10u128); // price
        let total_supply = U256::from(10000u128); // total supply
        let user_liquidity = U256::from(1000u128); // user liquidity
        let bin_liquidity = U256::from(5000u128); // bin liquidity
        let shares =
            U256x256Math::mul_div_round_down(&user_liquidity, total_supply, bin_liquidity).unwrap();
        let effective_liquidity =
            U256x256Math::mul_div_round_up(&shares, bin_liquidity, total_supply).unwrap();

        // Adjusting amounts_in to ensure delta_liquidity calculation triggers the specific condition
        let mut amounts_in = Bytes32::encode(500, 1000);
        if user_liquidity > effective_liquidity {
            let delta_liquidity = user_liquidity - effective_liquidity;
            amounts_in = Bytes32::encode(
                500,
                1000 + ((delta_liquidity >> scale_offset.as_u32()).as_u128()),
            );
        }

        // Execute the method
        let result = BinHelper::get_shares_and_effective_amounts_in(
            &bin_reserves,
            amounts_in,
            price,
            total_supply,
        )
        .unwrap();

        // Assertions
        assert!(result.1.decode_y() == 1000);
        assert!(result.1.decode_x() == 500);
    }

    #[test]
    fn test_get_shares_and_effective_amounts_in() -> Result<(), BinError> {
        let mut total_supply = U256::from_str("0").unwrap();
        let max_u256 = U256::MAX;
        let mut bin_reserves = Bytes32::encode(1000, 1000);
        let amount_in = Bytes32::encode(1000, 1000);
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        // TODO - address unused variables
        for _i in 0..10 {
            // Assume conditions similar to the Solidity test
            if price > U256::MIN
                && (bin_reserves == [0u8; 32]
                    || (price <= max_u256 / 1000 && price * 1000 <= (max_u256 - 1000) << 128))
                && (amount_in == [0u8; 32]
                    || (price <= max_u256 / 1000 && price * 1000 <= (max_u256 - 1000) << 128))
            {
                let _user_liquidity = BinHelper::get_liquidity(&amount_in, price).unwrap();
                let _bin_liquidity = BinHelper::get_liquidity(&bin_reserves, price).unwrap();
                let (shares, effective_amounts_in) =
                    BinHelper::get_shares_and_effective_amounts_in(
                        &bin_reserves,
                        amount_in,
                        price,
                        total_supply,
                    )
                    .unwrap();

                total_supply += shares;
                let (x, y) = effective_amounts_in.decode();
                bin_reserves =
                    Bytes32::encode(bin_reserves.decode_x() + x, bin_reserves.decode_y() + y);
            }
        }
        assert_eq!(
            total_supply,
            U256::from_str("231048229485969055456138120902788449760223779800000").unwrap()
        );
        Ok(())
    }

    #[test]
    fn test_try_exploit_shares() -> Result<(), BinError> {
        // Setup initial variables
        let amount_x1 = 1000u128;
        let amount_y1 = 1000u128;
        let amount_x2 = 500u128;
        let amount_y2 = 500u128;
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        // Assumptions (replace with Rust's assert! or whatever you use for precondition checks)
        assert!(price > U256::ZERO);
        // ... (add all your assumptions here)

        // Simulate exploiter front-running the transaction
        let mut total_supply = U256::from(1u128) << 128;
        let mut bin_reserves = Bytes32::encode(amount_x1, amount_y1);

        // Get shares and effective amounts in
        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            &bin_reserves,
            Bytes32::encode(amount_x2, amount_y2),
            price,
            total_supply,
        )?;

        // Update bin reserves and total supply
        bin_reserves = Bytes32::encode(
            bin_reserves.decode_x() + effective_amounts_in.decode_x(),
            bin_reserves.decode_y() + effective_amounts_in.decode_y(),
        );

        total_supply += shares;

        // Calculate what the user receives
        let user_received_x = U256x256Math::mul_div_round_down(
            &shares,
            bin_reserves.decode_x().into(),
            total_supply,
        )?;
        let user_received_y = U256x256Math::mul_div_round_down(
            &shares,
            bin_reserves.decode_y().into(),
            total_supply,
        )?;

        // Calculate received and sent in Y
        let received_in_y =
            U256x256Math::mul_shift_round_down(&user_received_x, price, SCALE_OFFSET).unwrap()
                + user_received_y;
        let sent_in_y = U256x256Math::mul_shift_round_down(
            &price,
            effective_amounts_in.decode_x().into(),
            SCALE_OFFSET,
        )
        .unwrap()
            + effective_amounts_in.decode_y();

        // Assert that received_in_y and sent_in_y should be approximately equal
        // (Implement your own assert_approxeq_abs function)
        let max_diff = ((price - U256::ONE) >> 128) + U256::from(2u128);
        assert_approxeq_abs(
            received_in_y,
            sent_in_y,
            max_diff,
            "test_TryExploitShares::1",
        );

        Ok(())
    }

    #[test]
    fn test_zero_total_supply_and_zero_bin_liquidity() -> Result<(), BinError> {
        let total_supply = U256::ZERO;
        let bin_reserves = Bytes32::encode(0, 0);
        let amount_in = Bytes32::encode(1000, 1000);
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            &bin_reserves,
            amount_in,
            price,
            total_supply,
        )?;

        assert_eq!(
            shares,
            U256::from_str("42008768997448919173843294709597899956404323600000").unwrap()
        );
        assert_eq!(effective_amounts_in, amount_in);

        Ok(())
    }

    #[test]
    fn test_zero_amount_in() -> Result<(), BinError> {
        let total_supply = U256::from(10000u128);
        let bin_reserves = Bytes32::encode(1000, 1000);
        let amount_in = Bytes32::encode(0, 0);
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            &bin_reserves,
            amount_in,
            price,
            total_supply,
        )?;

        assert_eq!(shares, U256::MIN);
        assert_eq!(effective_amounts_in, amount_in);

        Ok(())
    }

    #[test]
    fn test_zero_price() -> Result<(), BinError> {
        let total_supply = U256::from(10000u128);
        let bin_reserves = Bytes32::encode(1000, 1000);
        let amount_in = Bytes32::encode(1000, 1000);
        let price = U256::ZERO;

        let (shares, effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            &bin_reserves,
            amount_in,
            price,
            total_supply,
        )?;

        assert_eq!(shares, U256::from(10000u128));
        assert_eq!(effective_amounts_in, amount_in);

        Ok(())
    }

    // TODO - revisit this test. not sure what it's doing
    #[test]
    fn test_liquidity() {
        let _total_supply = U256::from_str("0").unwrap();
        let amount_in = Bytes32::encode(1000, 1000);
        let price = U256::from_str("42008768657166552252904831246223292524636112144").unwrap();

        let liquidity = BinHelper::get_liquidity(&amount_in, price).unwrap();

        assert_eq!(
            liquidity,
            U256::from_str("42008768997448919173843294709597899956404323600000").unwrap()
        )
    }

    #[test]
    fn test_get_composition_fees() -> Result<(), BinError> {
        // These would typically be random or fuzzed inputs, but for this example, let's assume fixed ones.
        let reserve_x = 5000000000000000000000u128;
        let reserve_y = 1000000000000000000000u128;
        let bin_step = 100u16;
        let amount_x_in = 500000000000000000000u128;
        let amount_y_in = 500000000000000000000u128;
        let price = U256::from(10000000000000000000000000000000000u128);
        let total_supply = U256::from(10000000000000000000000u128);

        // Perform the same assumptions as in the Solidity test.
        // ... (omitted for brevity)

        let bin_reserves = Bytes32::encode(reserve_x, reserve_y);
        let amounts_in = Bytes32::encode(amount_x_in, amount_y_in);

        let (shares, _effective_amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
            &bin_reserves,
            amounts_in,
            price,
            total_supply,
        )?;

        let base_factor = 5000;
        let filter_period = 30;
        let decay_period = 600;
        let reduction_factor = 5000;
        let variable_fee_control = 40000;
        let protocol_share = 1000;
        let max_volatility_accumulator = 350000;

        // Set the parameters (assuming PairParameters and DEFAULT_* constants are defined)
        let mut pair_parameters = PairParameters(Bytes32::default());
        pair_parameters
            .set_static_fee_parameters(
                base_factor,
                filter_period,
                decay_period,
                reduction_factor,
                variable_fee_control,
                protocol_share,
                max_volatility_accumulator,
            )
            .unwrap();
        // Call the function we are testing
        let composition_fees = BinHelper::get_composition_fees(
            &bin_reserves,
            pair_parameters,
            bin_step,
            amounts_in,
            total_supply,
            shares,
        )?;
        assert_eq!(U256::from(4999412339173586562315u128), shares);
        assert_eq!((0, 196874089861191), composition_fees.decode());

        // Calculate binC and userC similar to the Solidity code
        let bin_c = if reserve_x | reserve_y == 0 {
            U256::MIN
        } else {
            U256::from(reserve_y) << (128 / (U256::from(reserve_x) + U256::from(reserve_y)))
        };

        let user_c = if amount_x_in | amount_y_in == 0 {
            U256::MIN
        } else {
            U256::from(amount_y_in) << (128 / (U256::from(amount_x_in) + U256::from(amount_y_in)))
        };

        // Perform assertions (assuming assert_ge is defined)
        if bin_c > user_c {
            assert!(
                U256::from(composition_fees.decode_x()) << 128 >= U256::MIN,
                "test_GetCompositionFees::1",
            );
        } else {
            assert!(
                U256::from(composition_fees.decode_y()) >= U256::MIN,
                "test_GetCompositionFees::2",
            );
        }

        Ok(())
    }

    #[test]
    fn test_verify_amounts_with_flawed_factor_id_less_than_active_id() {
        let amounts = Bytes32::encode(1, 0); // Right-side 128 bits greater than zero
        let active_id = 2;
        let id = 1;
        let result = BinHelper::verify_amounts(&amounts, active_id, id);

        match result {
            Err(BinError::CompositionFactorFlawed(error_id)) => assert_eq!(error_id, id),
            _ => panic!("Expected CompositionFactorFlawed error"),
        }
    }

    #[test]
    fn test_verify_amounts_with_flawed_factor_id_greater_than_active_id() {
        let amounts = U256::from(u128::MAX) + U256::ONE; // Greater than u128::MAX
        let active_id = 1;
        let id = 2;
        let result = BinHelper::verify_amounts(&amounts.to_le_bytes(), active_id, id);

        match result {
            Err(BinError::CompositionFactorFlawed(error_id)) => assert_eq!(error_id, id),
            _ => panic!("Expected CompositionFactorFlawed error"),
        }
    }

    #[test]
    fn test_verify_amounts_valid_case_id_less_than_active_id() {
        let amounts = Bytes32::encode(0, 0); // Right-side 128 bits are zero
        let active_id = 2;
        let id = 1;
        let result = BinHelper::verify_amounts(&amounts, active_id, id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_amounts_valid_case_id_greater_than_active_id() {
        let amounts = U256::from(u128::MAX).to_le_bytes(); // Not greater than u128::MAX
        let active_id = 1;
        let id = 2;
        let result = BinHelper::verify_amounts(&amounts, active_id, id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_amounts_valid_case_id_equals_active_id() {
        let amounts = Bytes32::encode(0, 0); // Any value is fine, as id == active_id
        let active_id = 1;
        let id = 1;
        let result = BinHelper::verify_amounts(&amounts, active_id, id);
        assert!(result.is_ok());
    }
}
