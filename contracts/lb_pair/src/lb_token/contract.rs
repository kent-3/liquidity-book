// TODO: this file might be useless...

use super::{execute::*, query::*, Error, Result};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use liquidity_book::interfaces::lb_token2::*;

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
