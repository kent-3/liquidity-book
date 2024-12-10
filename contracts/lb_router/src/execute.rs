use crate::{
    contract::{SHADE_ROUTER_KEY, SWAP_REPLY_ID},
    error::LBRouterError,
    msg::{ExecuteMsgResponse, Hop, TokenAmount},
    state::{CurrentSwapInfo, CONFIG, EPHEMERAL_STORAGE},
};
use cosmwasm_std::{
    to_binary, Addr, Coin, ContractInfo, CosmosMsg, DepsMut, Env, Response, StdResult, SubMsg,
    Uint128, WasmMsg,
};
use lb_interfaces::lb_pair;
use shade_protocol::{
    snip20::helpers::{register_receive, set_viewing_key_msg},
    swap::core::TokenType,
};

pub fn create_lb_pair() {
    todo!()
}

pub fn add_liquidity() {
    todo!()
}

pub fn add_liquidity_native() {
    unimplemented!()
}

pub fn remove_liquidity() {
    todo!()
}

pub fn remove_liquidity_native() {
    unimplemented!()
}

pub fn swap_exact_tokens_for_tokens() {
    todo!()
}

pub fn swap_exact_tokens_for_native() {
    unimplemented!()
}

pub fn swap_exact_native_for_tokens() {
    unimplemented!()
}

/// Execute Swap for Exact Token
pub fn swap_tokens_for_exact_tokens(
    deps: DepsMut,
    env: Env,
    amount_in: TokenAmount,
    amount_out_min: Option<Uint128>,
    path: &Vec<Hop>,
    sender: Addr,
    recipient: Option<Addr>,
    mut response: Response,
) -> Result<Response, LBRouterError> {
    todo!()
}

pub fn swap_tokens_for_exact_native() {
    unimplemented!()
}
