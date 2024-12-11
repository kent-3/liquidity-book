use crate::{
    contract::{CREATE_LB_PAIR_REPLY_ID, ROUTER_KEY, SWAP_REPLY_ID},
    msg::ExecuteMsgResponse,
    prelude::*,
    state::FACTORY,
};
use cosmwasm_std::{
    to_binary, Addr, ContractInfo, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, Uint128,
    Uint256, Uint64,
};
use lb_interfaces::{
    lb_factory,
    lb_pair::{self, LiquidityParameters},
    lb_router::{self, Path},
};
use shade_protocol::{
    snip20::helpers::{register_receive, set_viewing_key_msg},
    swap::core::TokenType,
    utils::ExecuteCallback,
};

pub fn create_lb_pair(
    deps: DepsMut,
    env: Env,
    token_x: TokenType,
    token_y: TokenType,
    active_id: u32,
    bin_step: u16,
) -> Result<Response> {
    let factory = FACTORY.load(deps.storage)?;
    let msg = lb_factory::ExecuteMsg::CreateLbPair {
        token_x,
        token_y,
        active_id,
        bin_step,
        viewing_key: ROUTER_KEY.to_string(),
        entropy: "meh".to_string(),
    }
    .to_cosmos_msg(&factory, vec![])?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, CREATE_LB_PAIR_REPLY_ID)))
}

pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    liquidity_parameters: LiquidityParameters,
) -> Result<Response> {
    todo!()
}

pub fn add_liquidity_native() {
    unimplemented!()
}

pub fn remove_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_x: ContractInfo,
    token_y: ContractInfo,
    bin_step: u16,
    amount_x_min: Uint128,
    amount_y_min: Uint128,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
    deadline: Uint64,
) -> Result<Response> {
    todo!()
}

pub fn remove_liquidity_native() {
    unimplemented!()
}

pub fn swap_exact_tokens_for_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount_in: Uint256,
    amount_out_min: Uint256,
    path: Path,
    to: String,
    deadline: Uint64,
) -> Result<Response> {
    todo!()
}

pub fn swap_exact_tokens_for_native() {
    unimplemented!()
}

pub fn swap_exact_native_for_tokens() {
    unimplemented!()
}

pub fn swap_tokens_for_exact_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount_in: Uint256,
    amount_out_min: Uint256,
    path: Path,
    to: String,
    deadline: Uint64,
) -> Result<Response> {
    todo!()
}

pub fn swap_tokens_for_exact_native() {
    unimplemented!()
}

pub fn sweep(
    token: ContractInfo, // must be a snip20 token
    to: String,
    amount: Uint128,
) -> Result<Response> {
    todo!()
}

pub fn sweep_lb_token(
    token: ContractInfo, // must be an LbToken
    to: String,
    ids: Vec<u32>,
    amounts: Vec<Uint128>,
) -> Result<Response> {
    todo!()
}
