use crate::prelude::*;
use cosmwasm_std::Uint128;
use lb_libraries::{
    constants::SCALE_OFFSET, math::u256x256_math::U256x256Math, price_helper::PriceHelper,
};

// NOTE: We are following the joe-v2 versioning, starting from V2_2.

pub fn _get_v2_quote(
    amount: Uint128,
    active_id: u32,
    bin_step: u16,
    swap_for_y: bool,
) -> Result<Uint128> {
    if swap_for_y {
        let x = PriceHelper::get_price_from_id(active_id, bin_step)?;
        let y = ethnum::U256::new(amount.u128());
        let quote = U256x256Math::mul_shift_round_down(x, y, SCALE_OFFSET)?.as_u128();

        Ok(Uint128::from(quote))
    } else {
        let x = ethnum::U256::new(amount.u128());
        let y = PriceHelper::get_price_from_id(active_id, bin_step)?;
        let quote = U256x256Math::shift_div_round_down(x, SCALE_OFFSET, y)?.as_u128();

        Ok(Uint128::from(quote))
    }
}
