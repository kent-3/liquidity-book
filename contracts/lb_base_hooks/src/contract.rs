#![allow(unused)]

use crate::{execute::*, query::*};
use cosmwasm_std::{
    entry_point, to_binary, Binary, ContractInfo, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint256,
};
use lb_interfaces::lb_hooks::*;
use lb_libraries::Bytes32;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::new())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::OnHooksSet {
            hooks_parameters,
            on_hooks_set_data,
        } => on_hooks_set(deps, env, hooks_parameters, on_hooks_set_data),
        ExecuteMsg::BeforeSwap {
            sender,
            to,
            swap_for_y,
            amounts_in,
        } => before_swap(deps, env, sender, to, swap_for_y, amounts_in),
        ExecuteMsg::AfterSwap {
            sender,
            to,
            swap_for_y,
            amounts_out,
        } => after_swap(deps, env, sender, to, swap_for_y, amounts_out),
        ExecuteMsg::BeforeFlashLoan {
            sender,
            to,
            amounts,
        } => before_flash_loan(deps, env, sender, to, amounts),
        ExecuteMsg::AfterFlashLoan {
            sender,
            to,
            fees,
            fees_received,
        } => after_flash_loan(deps, env, sender, to, fees, fees_received),
        ExecuteMsg::BeforeMint {
            sender,
            to,
            liquidity_configs,
            amounts_received,
        } => before_mint(deps, env, sender, to, liquidity_configs, amounts_received),
        ExecuteMsg::AfterMint {
            sender,
            to,
            liquidity_configs,
            amounts_in,
        } => after_mint(deps, env, sender, to, liquidity_configs, amounts_in),
        ExecuteMsg::BeforeBurn {
            sender,
            from,
            to,
            ids,
            amounts_to_burn,
        } => before_burn(deps, env, sender, from, to, ids, amounts_to_burn),
        ExecuteMsg::AfterBurn {
            sender,
            from,
            to,
            ids,
            amounts_to_burn,
        } => after_burn(deps, env, sender, from, to, ids, amounts_to_burn),
        ExecuteMsg::BeforeBatchTransferFrom {
            sender,
            from,
            to,
            ids,
            amounts,
        } => before_batch_transfer_from(deps, env, sender, from, to, ids, amounts),
        ExecuteMsg::AfterBatchTransferFrom {
            sender,
            from,
            to,
            ids,
            amounts,
        } => after_batch_transfer_from(deps, env, sender, from, to, ids, amounts),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLbPair {} => to_binary(&get_lb_pair(deps)?),
        QueryMsg::IsLinked {} => to_binary(&is_linked(deps)?),
    }
    // .map_err(Error::CwErr)
}
