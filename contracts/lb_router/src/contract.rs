#![allow(unused)] // For beginning only.

use crate::{execute::*, msg::*, prelude::*, query::*, state::*};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdResult, SubMsgResult, Uint128,
};
use lb_interfaces::{
    lb_pair,
    lb_router::{self, CreateLbPairResponse, Path},
};
use lb_libraries::math::packed_u128_math::PackedUint128Math;

// TODO: How are we going to register this router contract to be able to receive every supported snip20 token?
// I guess we can add a new ExecuteMsg type for that purpose, but if we ever deploy a new router, we'll need to
// re-register allllll the tokens.

// TODO: Modify create_lb_pair to register_receiver for the two tokens.

// TODO: Implement the receive interface to support swaps.

pub const BLOCK_SIZE: usize = 256;
pub const ROUTER_KEY: &str = "lb_router";

pub const CREATE_LB_PAIR_REPLY_ID: u64 = 1u64;
pub const MINT_REPLY_ID: u64 = 2u64;
pub const BURN_REPLY_ID: u64 = 3u64;
pub const SWAP_REPLY_ID: u64 = 10u64;

// TODO: Need to be able to query the factory contract to check who the owner/admin is.
// if (msg.sender != Ownable(address(_factory)).owner()) revert LBRouter__NotFactoryOwner();
pub fn only_factory_owner(deps: Deps, env: Env, info: MessageInfo) -> Result<()> {
    // let factory_owner = deps.querier.query_wasm_smart(code_hash, contract_addr, lb_factory::QueryMsg::???)
    todo!();

    Ok(())
}

pub fn ensure(env: &Env, deadline: u64) -> Result<()> {
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
    _info: MessageInfo,
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
        } => remove_liquidity(
            deps,
            env,
            info,
            token_x,
            token_y,
            bin_step,
            amount_x_min,
            amount_y_min,
            ids,
            amounts,
            to,
            deadline,
        ),
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
        } => swap_exact_tokens_for_tokens(
            deps,
            env,
            info,
            amount_in,
            amount_out_min,
            path,
            to,
            deadline,
        ),
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
        } => swap_tokens_for_exact_tokens(
            deps,
            env,
            info,
            amount_out,
            amount_in_max,
            path,
            to,
            deadline,
        ),
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
        ExecuteMsg::Sweep { token, to, amount } => sweep(deps, env, info, token, to, amount),
        ExecuteMsg::SweepLbToken {
            token,
            to,
            ids,
            amounts,
        } => sweep_lb_token(deps, env, info, token, to, ids, amounts),
        // TODO: Are these necessary?
        ExecuteMsg::RegisterSnip20 {
            token_addr,
            token_code_hash,
        } => unimplemented!(),
        ExecuteMsg::Receive { from, msg, amount } => unimplemented!(),
    }
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response> {
    match (msg.id.into(), msg.result) {
        (CREATE_LB_PAIR_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(data) => {
                let lb_pair: lb_pair::LbPair = from_binary(&data)?;
                // TODO: does it make sense to return a nested response like this that only
                // contains one value?
                let data = CreateLbPairResponse { lb_pair };

                Ok(Response::new().set_data(to_binary(&data)?))
            }
            None => Err(Error::ReplyDataMissing),
        },
        (MINT_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(data) => {
                let lb_pair::MintResponse {
                    amounts_received,
                    amounts_left,
                    liquidity_minted,
                } = from_binary(&data)?;
                let liq = EPHEMERAL_ADD_LIQUIDITY.load(deps.storage)?;

                let amounts_added = amounts_received.sub(amounts_left);

                let amount_x_added = Uint128::from(amounts_added.decode_x());
                let amount_y_added = Uint128::from(amounts_added.decode_y());

                if amount_x_added < liq.amount_x_min || amount_y_added < liq.amount_y_min {
                    return Err(Error::AmountSlippageCaught {
                        amount_x_min: liq.amount_x_min,
                        amount_x: amount_x_added,
                        amount_y_min: liq.amount_y_min,
                        amount_y: amount_y_added,
                    });
                }

                let amount_x_left = Uint128::from(amounts_left.decode_x());
                let amount_y_left = Uint128::from(amounts_left.decode_y());

                let data = lb_router::AddLiquidityResponse {
                    amount_x_added: amount_x_added.into(),
                    amount_y_added: amount_y_added.into(),
                    amount_x_left: amount_x_left.into(),
                    amount_y_left: amount_y_left.into(),
                    deposit_ids: liq.deposit_ids,
                    liquidity_minted,
                };

                // TODO: Decide between response attributes vs response data.
                Ok(
                    Response::new()
                        .set_data(to_binary(&data)?)
                        .add_attribute("amount_x_added", amount_x_added)
                        .add_attribute("amount_y_added", amount_y_added)
                        .add_attribute("amount_x_left", amount_x_left)
                        .add_attribute("amount_y_left", amount_y_left), // .add_attribute("deposit_ids", deposit_ids)
                                                                        // .add_attribute("liquidity_minted", liquidity_minted)
                )
            }
            None => Err(Error::ReplyDataMissing),
        },
        (BURN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(data) => {
                let lb_pair::BurnResponse {
                    amounts: amounts_burned,
                } = from_binary(&data)?;
                let liq = EPHEMERAL_REMOVE_LIQUIDITY.load(deps.storage)?;

                // let mut amount_x_burned: Uint128 = Uint128::zero();
                // let mut amount_y_burned: Uint128 = Uint128::zero();
                //
                // for amount_burned in amounts_burned {
                //     amount_x_burned += Uint128::from(amount_burned.decode_x());
                //     amount_y_burned += Uint128::from(amount_burned.decode_y());
                // }

                let mut amount_x_burned = 0u128;
                let mut amount_y_burned = 0u128;

                for amount_burned in amounts_burned {
                    amount_x_burned += amount_burned.decode_x();
                    amount_y_burned += amount_burned.decode_y();
                }

                let amount_x_burned = Uint128::from(amount_x_burned);
                let amount_y_burned = Uint128::from(amount_y_burned);

                if amount_x_burned < liq.amount_x_min || amount_y_burned < liq.amount_y_min {
                    return Err(Error::AmountSlippageCaught {
                        amount_x_min: liq.amount_x_min,
                        amount_x: amount_x_burned,
                        amount_y_min: liq.amount_y_min,
                        amount_y: amount_y_burned,
                    });
                }

                let data = lb_router::RemoveLiquidityResponse {
                    amount_x: amount_x_burned,
                    amount_y: amount_y_burned,
                };

                let response = Response::new().set_data(to_binary(&data)?);

                Ok(response)
            }
            None => Err(Error::ReplyDataMissing),
        },
        (SWAP_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(data) => {
                todo!()
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
