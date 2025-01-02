use crate::{execute::*, query::*, Error, Result};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};
use liquidity_book::interfaces::lb_token2::*;

#[entry_point]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    todo!()
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    match msg {
        ExecuteMsg::ApproveForAll { spender, approved } => {
            approve_for_all(deps, env, info, spender, approved)
        }
        ExecuteMsg::BatchTransferFrom {
            from,
            to,
            ids,
            amounts,
        } => batch_transfer_from(deps, env, info, from, to, ids, amounts),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::Name => to_binary(&query_name(deps)?),
        QueryMsg::Symbol => to_binary(&query_symbol(deps)?),
        QueryMsg::TotalSupply { id } => to_binary(&query_total_supply(deps, id)?),
        QueryMsg::BalanceOf { account, id } => to_binary(&query_balance_of(deps, account, id)?),
        QueryMsg::BalanceOfBatch { accounts, ids } => {
            to_binary(&query_balance_of_batch(deps, accounts, ids)?)
        }
        QueryMsg::IsApprovedForAll { owner, spender } => {
            to_binary(&query_is_approved_for_all(deps, owner, spender)?)
        }
    }
    .map_err(Error::StdError)
}
