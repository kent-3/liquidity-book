//! ## Liquidity Book U128x128 Math Library
//!
//! Author: Kent
//!
//! Helper library used for power and log calculations.

use super::bit_math::BitMath;
use crate::libraries::constants::*;
use ethnum::{I256, U256};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum U128x128MathError {
    #[error("U128x128 Math Error: LogUnderflow")]
    LogUnderflow,
    #[error("U128x128 Math Error: PowUnderflow {0} {1}")]
    PowUnderflow(U256, I256),
    #[error("U128x128 Math Error: ID Shift overflow")]
    IdShiftOverflow,
}

// This is 127
const LOG_SCALE_OFFSET: U256 = U256::new(127u128);
// This is 2^127 = 170141183460469231731687303715884105728
const LOG_SCALE: U256 = U256::new(1u128 << LOG_SCALE_OFFSET.as_u128());
// This is 2^254 = 28948022309329048855892746252171976963317496166410141009864396001978282409984
const LOG_SCALE_SQUARED: U256 = U256::from_words(1u128 << (LOG_SCALE_OFFSET.as_u128() - 1), 0);

pub trait U128x128Math {
    /// Calculates the binary logarithm of x.
    fn log2(&self) -> Result<I256, U128x128MathError>;

    /// Returns the value of x^y. It calculates `1 / x^abs(y)` if x is bigger than 2^128.
    fn pow(&self, y: I256) -> Result<U256, U128x128MathError>;
}

impl U128x128Math for U256 {
    /// Calculates the binary logarithm of x.
    ///
    /// Based on the iterative approximation algorithm.
    /// <https://en.wikipedia.org/wiki/Binary_logarithm#Iterative_approximation>
    ///
    /// # Requirements
    ///
    /// - x must be greater than zero.
    ///
    /// # Caveats
    ///
    /// - The results are not perfectly accurate to the last decimal, due to the lossy precision of the iterative approximation.
    ///   Also because x is converted to an unsigned 129.127-binary fixed-point number during the operation to optimize the multiplication.
    ///
    /// # Arguments
    ///
    /// * `x` - The unsigned 128.128-binary fixed-point number for which to calculate the binary logarithm.
    ///
    /// # Returns
    ///
    /// * `result` - The binary logarithm as a signed 128.128-binary fixed-point number.
    fn log2(&self) -> Result<I256, U128x128MathError> {
        // Convert x to a unsigned 129.127-binary fixed-point number to optimize the multiplication.
        // If we use an offset of 128 bits, y would need 129 bits and y**2 would would overflow and we would have to
        // use mulDiv, by reducing x to 129.127-binary fixed-point number we assert that y will use 128 bits, and we
        // can use the regular multiplication

        let mut x = *self;

        if x == 1 {
            return Ok(I256::from(-128));
        }
        if x == 0 {
            return Err(U128x128MathError::LogUnderflow);
        }

        x >>= 1;

        // This works because log2(x) = -log2(1/x).
        let sign: I256;
        if x >= LOG_SCALE {
            sign = I256::ONE;
        } else {
            sign = I256::MINUS_ONE;
            // Do the fixed-point inversion inline to save gas
            x = LOG_SCALE_SQUARED / x;
        }

        // Calculate the integer part of the logarithm and add it to the result and finally calculate y = x * 2^(-n).
        let n = BitMath::most_significant_bit(x >> LOG_SCALE_OFFSET);

        // The integer part of the logarithm as a signed 129.127-binary fixed-point number. The operation can't overflow
        // because n is maximum 255, LOG_SCALE_OFFSET is 127 bits and sign is either 1 or -1.
        let mut result = I256::from(n) << LOG_SCALE_OFFSET;

        // This is y = x * 2^(-n).
        let mut y = x >> n;

        // If y = 1, the fractional part is zero.
        if y != LOG_SCALE {
            // Calculate the fractional part via the iterative approximation.
            // The "delta >>= 1" part is equivalent to "delta /= 2", but shifting bits is faster.
            let mut delta = I256::ONE << (LOG_SCALE_OFFSET - 1);
            while delta > 0 {
                y = (y * y) >> LOG_SCALE_OFFSET;

                // Is y^2 > 2 and so in the range [2,4)?
                if y >= U256::ONE << (LOG_SCALE_OFFSET + 1) {
                    // Add the 2^(-m) factor to the logarithm.
                    result += delta;

                    // Corresponds to z/2 on Wikipedia.
                    y >>= 1;
                }

                delta >>= 1;
            }
        }

        // Convert x back to unsigned 128.128-binary fixed-point number
        result = (result * sign) << 1;

        Ok(result)
    }

    /// Returns the value of x^y. It calculates `1 / x^abs(y)` if x is bigger than 2^128.
    ///
    /// At the end of the operations, we invert the result if needed.
    ///
    /// # Arguments
    ///
    /// * `x` - The unsigned 128.128-binary fixed-point number for which to calculate the power
    /// * `y` - A relative number without any decimals, needs to be between ]2^21; 2^21[
    fn pow(&self, y: I256) -> Result<U256, U128x128MathError> {
        let mut invert = false;
        let abs_y = y.abs().as_u128();

        if y == 0 {
            return Ok(SCALE);
        }

        if y < 0 {
            invert = !invert;
        }

        let mut result = SCALE;

        if abs_y < 0x100000 {
            let mut squared = *self;
            if *self > U256::from(0xffffffffffffffffffffffffffffffffu128) {
                squared = U256::MAX / squared;
                invert = !invert;
            }

            for i in 0..20 {
                if (abs_y & (1 << i)) != 0 {
                    result = (result * squared) >> 128;
                }
                squared = (squared * squared) >> 128;
            }
        }

        // revert if y is too big or if x^y underflowed
        if result == 0 {
            return Err(U128x128MathError::PowUnderflow(*self, y));
        }

        if invert {
            Ok(U256::MAX / result)
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use ethnum::{AsI256, U256};

    #[test]
    fn test_pow() {
        let x: U256 = (U256::from((1.0001 * PRECISION as f64) as u128) << 128) / PRECISION;
        let y = 100_000;
        let res = U128x128Math::pow(&x, y.into()).unwrap();
        // TODO: I don't think bitxor is what was intended here
        let tolerance = 10 ^ 12;

        let expected = U256::from_str("7491471493045233295460405875225305845649644").unwrap();

        assert!(
            res > expected - tolerance && res < expected + tolerance,
            "test_Pow::1 failed"
        );
    }

    #[test]
    fn test_pow_and_log() {
        let x: U256 = (U256::from((1.0001 * PRECISION as f64) as u128) << 128) / PRECISION;
        let y = 100_000;
        let res = U128x128Math::pow(&x, y.into()).unwrap();
        let tolerance = 10 ^ 12;

        let expected = U256::from_str("7491471493045233295460405875225305845649644").unwrap();

        assert!(
            res > expected - tolerance && res < expected + tolerance,
            "test_Pow::1 failed"
        );

        let base_log2 = U128x128Math::log2(&x).unwrap();

        assert_eq!(
            base_log2,
            I256::from_str("49089913871092318234424474366155884").unwrap()
        );
        let res = U128x128Math::log2(&res).unwrap() / base_log2;
        let expected = 100000;

        assert_eq!(res, I256::from_str("100000").unwrap());

        assert!(
            res > expected.as_i256() - tolerance.as_i256()
                && res < expected.as_i256() + tolerance.as_i256(),
            "test_pow_and_log::1 failed"
        );
    }
    #[test]
    fn test_log2_x_equals_1() {
        let x = U256::ONE;
        let result = U128x128Math::log2(&x).unwrap();
        assert_eq!(result, I256::from(-128));
    }

    #[test]
    fn test_log2_x_equals_0() {
        let x = U256::ZERO;
        let result = U128x128Math::log2(&x);

        assert!(matches!(result, Err(U128x128MathError::LogUnderflow)));
    }

    #[test]
    fn test_pow_y_equals_0() {
        let x = U256::from(123456789u128);
        let y = I256::ZERO;
        let result = U128x128Math::pow(&x, y).unwrap();
        assert_eq!(result, SCALE);
    }

    #[test]
    fn test_pow_y_negative() {
        let x = U256::from(123456789u128);
        let y = I256::from(-1);
        let _result = U128x128Math::pow(&x, y).unwrap();
        assert_eq!(
            _result.to_string(),
            "937915931357296158282320018939484225960793332034907890237268231623237"
        );
    }

    #[test]
    fn test_pow_y_positive() {
        let x = U256::from(123456789u128);
        let y = I256::from(1);
        let _result = U128x128Math::pow(&x, y).unwrap();
        assert_eq!(_result.to_string(), "123456789");
    }

    #[test]
    fn test_pow_invert_result() {
        let x = U256::from(0xffffffffffffffffffffffffffffffffu128);
        let y = I256::from(-1);
        let _result = U128x128Math::pow(&x, y).unwrap();
        assert_eq!(
            _result.to_string(),
            "340282366920938463463374607431768211457"
        );
    }
}
