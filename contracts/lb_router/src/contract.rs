#![allow(unused)] // For beginning only.

use crate::{
    execute::{
        add_liquidity, create_lb_pair, remove_liquidity, swap_exact_tokens_for_tokens,
        swap_tokens_for_exact_tokens, sweep, sweep_lb_token,
    },
    msg::*,
    prelude::*,
    query::*,
    state::*,
};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, ContractInfo, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, SubMsgResult, Timestamp, Uint128,
    Uint256, WasmMsg,
};
use ethnum::U256;
use lb_interfaces::{
    lb_factory,
    lb_router::{Path, Version},
};
use lb_libraries::{
    bin_helper::BinHelper,
    math::{encoded::Encoded, packed_u128_math::PackedUint128Math, u24::U24},
    types::{Bytes32, LiquidityConfigurations},
};
use shade_protocol::{contract_interfaces::swap::core::TokenType, utils::ExecuteCallback};

const BLOCK_SIZE: usize = 256;
pub const ROUTER_KEY: &str = "lb_router";
pub const CREATE_LB_PAIR_REPLY_ID: u64 = 1u64;
pub const MINT_REPLY_ID: u64 = 99u64;
pub const SWAP_REPLY_ID: u64 = 2u64;

// TODO: Need to be able to query the factory contract to check who the owner/admin is.
// if (msg.sender != Ownable(address(_factory)).owner()) revert LBRouter__NotFactoryOwner();
pub fn only_factory_owner(deps: Deps, env: Env, info: MessageInfo) -> Result<()> {
    // let factory_owner = deps.querier.query_wasm_smart(code_hash, contract_addr, lb_factory::QueryMsg::???)
    todo!();

    Ok(())
}

pub fn ensure(env: Env, deadline: u64) -> Result<()> {
    if env.block.time.seconds() > deadline {
        return Err(Error::DeadlineExceeded {
            deadline,
            timestamp: env.block.time.seconds(),
        });
    }

    Ok(())
}

pub fn verify_path_validity(path: &Path) -> Result<()> {
    if path.pair_bin_steps.is_empty()
        || path.versions.len() != path.pair_bin_steps.len()
        || path.pair_bin_steps.len() + 1 != path.token_path.len()
    {
        return Err(Error::LengthsMismatch);
    }
    Ok(())
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    FACTORY.save(deps.storage, &msg.factory)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    match msg {
        ExecuteMsg::CreateLbPair {
            token_x,
            token_y,
            active_id,
            bin_step,
        } => create_lb_pair(deps, env, token_x, token_y, active_id, bin_step),
        ExecuteMsg::AddLiquidity {
            liquidity_parameters,
        } => add_liquidity(deps, env, info, liquidity_parameters),
        ExecuteMsg::AddLiquidityNative {
            liquidity_parameters,
        } => unimplemented!(),
        ExecuteMsg::RemoveLiquidity {
            token_x,
            token_y,
            bin_step,
            amount_x_min,
            amount_y_min,
            ids,
            amounts,
            to,
            deadline,
        } => todo!(),
        ExecuteMsg::RemoveLiquidityNative {
            token,
            bin_step,
            amount_token_min,
            amount_native_min,
            ids,
            amounts,
            to,
            deadline,
        } => unimplemented!(),
        ExecuteMsg::SwapExactTokensForTokens {
            amount_in,
            amount_out_min,
            path,
            to,
            deadline,
        } => todo!(),
        ExecuteMsg::SwapExactTokensForNative {
            amount_in,
            amount_out_min_native,
            path,
            to,
            deadline,
        } => unimplemented!(),
        ExecuteMsg::SwapExactNativeforTokens {
            amount_out_min,
            path,
            to,
            deadline,
        } => unimplemented!(),
        ExecuteMsg::SwapTokensForExactTokens {
            amount_out,
            amount_in_max,
            path,
            to,
            deadline,
        } => todo!(),
        ExecuteMsg::SwapTokensForExactNative {
            amount_native_out,
            amount_in_max,
            path,
            to,
            deadline,
        } => unimplemented!(),
        ExecuteMsg::SwapNativeforExactTokens {
            amount_out,
            path,
            to,
            deadline,
        } => unimplemented!(),
        ExecuteMsg::SwapExactTokensForTokensSupportingFeeOnTransferTokens {
            amount_in,
            amount_out_min,
            path,
            to,
            deadline,
        } => unimplemented!(),
        ExecuteMsg::SwapExactTokensForNativesupportingFeeOnTransferTokens {
            amount_in,
            amount_out_min_native,
            path,
            to,
            deadline,
        } => unimplemented!(),
        ExecuteMsg::SwapExactNativeforTokensSupportingFeeOnTransferTokens {
            amount_out_min,
            path,
            to,
            deadline,
        } => unimplemented!(),
        ExecuteMsg::Sweep { token, to, amount } => todo!(),
        ExecuteMsg::SweepLbToken {
            token,
            to,
            ids,
            amounts,
        } => todo!(),
        // TODO: Are these necessary?
        ExecuteMsg::RegisterSnip20 {
            token_addr,
            token_code_hash,
        } => todo!(),
        ExecuteMsg::Receive { from, msg, amount } => todo!(),
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response> {
    match (msg.id.into(), msg.result) {
        (CREATE_LB_PAIR_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                // TODO: create a response type for CreateLbPair that returns the contract address
                // and whatever else about the newly created pair.
                // let data: lb_factory::CreateLbPairResponse = from_binary(&x)?;

                let data = [0u8; 32];

                Ok(Response::new().set_data(to_binary(&data)?))
            }
            None => Err(Error::ReplyDataMissing),
        },
        (MINT_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                // let data: lb_pair::??? = from_binary(&x)?;
                //
                // let amount_x_added = Uint128::from(data.amounts_received.decode_x());
                // let amount_y_added = Uint128::from(data.amounts_received.decode_y());
                //
                // let amount_x_left = Uint128::from(data.amounts_left.decode_x());
                // let amount_y_left = Uint128::from(data.amounts_left.decode_y());
                // let deposit_ids = serde_json_wasm::to_string(&data.deposit_ids);
                // let liquidity_minted = serde_json_wasm::to_string(&data.liquidity_minted);

                Ok(Response::new())
                // .add_attribute("amount_x_added", amount_x_added)
                // .add_attribute("amount_y_added", amount_y_added)
                // .add_attribute("amount_x_left", amount_x_left)
                // .add_attribute("amount_y_left", amount_y_left)
                // .add_attribute("liquidity_minted", liquidity_minted.unwrap())
                // .add_attribute("deposit_ids", deposit_ids.unwrap()))
            }
            None => Err(Error::ReplyDataMissing),
        },
        _ => Err(Error::UnknownReplyId { id: msg.id }),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetFactory {} => to_binary(&query_factory(deps)?),
        QueryMsg::GetIdFromPrice { lb_pair, price } => {
            to_binary(&query_id_from_price(deps, lb_pair, price)?)
        }
        QueryMsg::GetPriceFromId { lb_pair, id } => {
            to_binary(&query_price_from_id(deps, lb_pair, id)?)
        }
        QueryMsg::GetSwapIn {
            lb_pair,
            amount_out,
            swap_for_y,
        } => to_binary(&query_swap_in(deps, lb_pair, amount_out, swap_for_y)?),
        QueryMsg::GetSwapOut {
            lb_pair,
            amount_in,
            swap_for_y,
        } => to_binary(&query_swap_out(deps, lb_pair, amount_in, swap_for_y)?),
    }
    .map_err(Error::CwErr)
}
