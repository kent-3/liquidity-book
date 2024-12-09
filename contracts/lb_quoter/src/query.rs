use crate::{prelude::*, state::*};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Decimal, Deps, Env, StdResult, Uint128, Uint256,
};
use lb_interfaces::{lb_factory::*, lb_pair::*, lb_quoter::*, lb_router::*};
use lb_libraries::{
    bin_helper::BinHelper,
    constants::SCALE_OFFSET,
    fee_helper::FeeHelper,
    math::{
        packed_u128_math::PackedUint128Math,
        u24::U24,
        u256x256_math::U256x256Math,
        uint256_to_u256::{ConvertU256, ConvertUint256},
    },
    oracle_helper::{self, OracleMap},
    price_helper::PriceHelper,
    types::Bytes32,
};

pub fn find_best_path_from_amount_in(deps: Deps) -> Result<Quote> {
    todo!()
}

pub fn find_best_path_from_amount_out(deps: Deps) -> Result<Quote> {
    todo!()
}
