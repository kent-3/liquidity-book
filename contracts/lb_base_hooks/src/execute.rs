#![allow(unused)]

use crate::{state::LB_PAIR, Error, Result};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256};
use liquidity_book::libraries::{hooks::HooksParameters, Bytes32};

pub fn only_trusted_caller(deps: Deps, info: MessageInfo) -> Result<()> {
    _check_trusted_caller(deps, info)
}

pub fn on_hooks_set(
    deps: DepsMut,
    env: Env,
    hooks_parameters: HooksParameters,
    on_hooks_set_data: Option<Binary>,
) -> Result<Response> {
    // Implementation for OnHooksSet
    Ok(Response::default())
}

pub fn before_swap(
    deps: DepsMut,
    env: Env,
    sender: String,
    to: String,
    swap_for_y: bool,
    amounts_in: Bytes32,
) -> Result<Response> {
    // Implementation for BeforeSwap
    Ok(Response::default())
}

pub fn after_swap(
    deps: DepsMut,
    env: Env,
    sender: String,
    to: String,
    swap_for_y: bool,
    amounts_out: Bytes32,
) -> Result<Response> {
    // Implementation for AfterSwap
    Ok(Response::default())
}

pub fn before_flash_loan(
    deps: DepsMut,
    env: Env,
    sender: String,
    to: String,
    amounts: Bytes32,
) -> Result<Response> {
    // Implementation for BeforeFlashLoan
    Ok(Response::default())
}

pub fn after_flash_loan(
    deps: DepsMut,
    env: Env,
    sender: String,
    to: String,
    fees: Bytes32,
    fees_received: Bytes32,
) -> Result<Response> {
    // Implementation for AfterFlashLoan
    Ok(Response::default())
}

pub fn before_mint(
    deps: DepsMut,
    env: Env,
    sender: String,
    to: String,
    liquidity_configs: Vec<Bytes32>,
    amounts_received: Bytes32,
) -> Result<Response> {
    // Implementation for BeforeMint
    Ok(Response::default())
}

pub fn after_mint(
    deps: DepsMut,
    env: Env,
    sender: String,
    to: String,
    liquidity_configs: Vec<Bytes32>,
    amounts_in: Bytes32,
) -> Result<Response> {
    // Implementation for AfterMint
    Ok(Response::default())
}

pub fn before_burn(
    deps: DepsMut,
    env: Env,
    sender: String,
    from: String,
    to: String,
    ids: Vec<u32>,
    amounts_to_burn: Vec<Uint256>,
) -> Result<Response> {
    // Implementation for BeforeBurn
    Ok(Response::default())
}

pub fn after_burn(
    deps: DepsMut,
    env: Env,
    sender: String,
    from: String,
    to: String,
    ids: Vec<u32>,
    amounts_to_burn: Vec<Uint256>,
) -> Result<Response> {
    // Implementation for AfterBurn
    Ok(Response::default())
}

pub fn before_batch_transfer_from(
    deps: DepsMut,
    env: Env,
    sender: String,
    from: String,
    to: String,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> Result<Response> {
    // Implementation for BeforeBatchTransferFrom
    Ok(Response::default())
}

pub fn after_batch_transfer_from(
    deps: DepsMut,
    env: Env,
    sender: String,
    from: String,
    to: String,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> Result<Response> {
    // Implementation for AfterBatchTransferFrom
    Ok(Response::default())
}

/// Checks that the caller is the LB Pair, otherwise reverts.
pub fn _check_trusted_caller(deps: Deps, info: MessageInfo) -> Result<()> {
    if let Some(lb_pair) = LB_PAIR.load(deps.storage)? {
        if info.sender != lb_pair.address {
            return Err(Error::InvalidCaller(info.sender));
        }
    }

    Ok(())
}
