use crate::{msg::*, prelude::*, state::CONFIG};
use cosmwasm_std::{
    to_binary, ContractInfo, Deps, QuerierWrapper, QueryRequest, StdResult, Uint128, Uint256,
    WasmQuery,
};
use lb_interfaces::lb_pair;

// pub fn pair_contract_config(
//     querier: &QuerierWrapper,
//     pair_contract_address: ContractInfo,
// ) -> StdResult<TokensResponse> {
//     let result: lb_pair::TokensResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
//         contract_addr: pair_contract_address.address.to_string(),
//         code_hash: pair_contract_address.code_hash,
//         msg: to_binary(&lb_pair::QueryMsg::GetTokens {})?,
//     }))?;
//
//     Ok(result)
// }

pub fn query_factory(deps: Deps) -> Result<FactoryResponse> {
    let state = CONFIG.load(deps.storage)?;
    Ok(FactoryResponse {
        factory: state.factory.address,
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
            &(&msg),
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
            &(&msg),
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
        &(&msg),
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
        &(&msg),
    )?;

    Ok(SwapOutResponse {
        amount_in_left,
        amount_out,
        fee,
    })
}
