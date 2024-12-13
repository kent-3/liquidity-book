use crate::{prelude::*, state::FACTORY};
use cosmwasm_std::{ContractInfo, Deps, Uint128, Uint256};
use lb_interfaces::lb_pair::{
    self, FactoryResponse, IdFromPriceResponse, PriceFromIdResponse, SwapInResponse,
    SwapOutResponse,
};

pub fn query_factory(deps: Deps) -> Result<FactoryResponse> {
    let factory = FACTORY.load(deps.storage)?;

    Ok(FactoryResponse {
        factory: factory.address,
    })
}

pub fn query_id_from_price(
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

pub fn query_price_from_id(
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

pub fn query_swap_in(
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

pub fn query_swap_out(
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
