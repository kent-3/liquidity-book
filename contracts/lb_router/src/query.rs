use crate::{state::FACTORY_V2_2, Result};
use cosmwasm_std::{ContractInfo, Deps, Uint128, Uint256};
use liquidity_book::interfaces::lb_pair::{
    self, FactoryResponse, IdFromPriceResponse, PriceFromIdResponse, SwapInResponse,
    SwapOutResponse,
};

/// Get the factory address.
pub fn get_factory(deps: Deps) -> Result<FactoryResponse> {
    let factory = FACTORY_V2_2.load(deps.storage)?;

    Ok(FactoryResponse {
        factory: factory.address.clone(),
    })
}

/// Returns the approximate id corresponding to the inputted price.
///
/// Warning, the returned id may be inaccurate close to the start price of a bin.
pub fn get_id_from_price(
    deps: Deps,
    lb_pair: ContractInfo,
    price: Uint256,
) -> Result<IdFromPriceResponse> {
    let msg = lb_pair::QueryMsg::GetIdFromPrice { price };
    let lb_pair::IdFromPriceResponse { id } = deps
        .querier
        .query_wasm_smart::<lb_pair::IdFromPriceResponse>(
            lb_pair.code_hash,
            lb_pair.address.to_string(),
            &msg,
        )?;

    Ok(IdFromPriceResponse { id })
}

/// Returns the price corresponding to the inputted id.
pub fn get_price_from_id(
    deps: Deps,
    lb_pair: ContractInfo,
    id: u32,
) -> Result<PriceFromIdResponse> {
    let msg = lb_pair::QueryMsg::GetPriceFromId { id };
    let lb_pair::PriceFromIdResponse { price } = deps
        .querier
        .query_wasm_smart::<lb_pair::PriceFromIdResponse>(
            lb_pair.code_hash,
            lb_pair.address.to_string(),
            &msg,
        )?;

    Ok(PriceFromIdResponse { price })
}

/// Simulate a swap in.
pub fn get_swap_in(
    deps: Deps,
    lb_pair: ContractInfo,
    amount_out: Uint128,
    swap_for_y: bool,
) -> Result<SwapInResponse> {
    let msg = lb_pair::QueryMsg::GetSwapIn {
        amount_out,
        swap_for_y,
    };
    let lb_pair::SwapInResponse {
        amount_in,
        amount_out_left,
        fee,
    } = deps.querier.query_wasm_smart::<lb_pair::SwapInResponse>(
        lb_pair.code_hash,
        lb_pair.address.to_string(),
        &msg,
    )?;

    Ok(SwapInResponse {
        amount_in,
        amount_out_left,
        fee,
    })
}

/// Simulate a swap out.
pub fn get_swap_out(
    deps: Deps,
    lb_pair: ContractInfo,
    amount_in: Uint128,
    swap_for_y: bool,
) -> Result<SwapOutResponse> {
    let msg = lb_pair::QueryMsg::GetSwapOut {
        amount_in,
        swap_for_y,
    };
    let lb_pair::SwapOutResponse {
        amount_in_left,
        amount_out,
        fee,
    } = deps.querier.query_wasm_smart::<lb_pair::SwapOutResponse>(
        lb_pair.code_hash,
        lb_pair.address.to_string(),
        &msg,
    )?;

    Ok(SwapOutResponse {
        amount_in_left,
        amount_out,
        fee,
    })
}
