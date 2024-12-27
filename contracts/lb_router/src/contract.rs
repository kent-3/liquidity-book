// #![allow(unused)] // For beginning only.

use crate::{execute::*, msg::*, prelude::*, query::*, state::*};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdResult, SubMsgResult, Uint128,
};
use lb_interfaces::{
    lb_factory::ILbFactory,
    lb_pair,
    lb_router::{self, CreateLbPairResponse, Path},
};
use lb_libraries::{math::packed_u128_math::PackedUint128Math, Bytes32};
use secret_toolkit::snip20;

// TODO: How are we going to register this router contract to be able to receive every supported snip20 token?
// I guess we can add a new ExecuteMsg type for that purpose, but if we ever deploy a new router, we'll need to
// re-register allllll the tokens.

// TODO: Modify create_lb_pair to register_receiver for the two tokens.

// TODO: Implement the receive interface to support swaps.

pub const BLOCK_SIZE: usize = 256;
// TODO: should the router contract have a viewing key that's used for every create_lb_pair? or
// should that belong to the factory?
pub const PUBLIC_VIEWING_KEY: &str = "lb_rocks";

pub const CREATE_LB_PAIR_REPLY_ID: u64 = 1u64;
pub const MINT_REPLY_ID: u64 = 2u64;
pub const BURN_REPLY_ID: u64 = 3u64;
pub const SWAP_REPLY_ID: u64 = 10u64;

// TODO: Need to be able to query the factory contract to check who the owner/admin is.
// Is there a way to find out the owner of a contract at the chain-level?
// if (msg.sender != Ownable(address(_factory2_2)).owner()) revert LBRouter__NotFactoryOwner();
pub fn only_factory_owner(deps: Deps, env: Env, info: MessageInfo) -> Result<()> {
    // let factory_owner = deps.querier.query_wasm_smart(code_hash, contract_addr, lb_factory::QueryMsg::???)
    todo!()
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
    FACTORY.save(deps.storage, &ILbFactory(msg.factory))?;

    // TODO: Register existing tokens with the router contract. If we ever deploy a new
    // router, we'll need a way to register all of the tokens used by the pairs.

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

        // not in joe-v2
        ExecuteMsg::Register { address, code_hash } => register(deps, env, address, code_hash),
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            memo,
            msg,
        } => receive(deps, env, info, sender, from, amount, memo, msg),
    }
}

pub fn register(deps: DepsMut, env: Env, address: String, code_hash: String) -> Result<Response> {
    deps.api.addr_validate(&address)?;

    let msg = snip20::register_receive_msg(
        env.contract.code_hash,
        None,
        1,
        address.to_string(),
        code_hash,
    )?;

    Ok(Response::new().add_message(msg))
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    mut info: MessageInfo, // TODO: does this work?
    _sender: Addr,
    from: Addr,
    _amount: Uint128,
    _memo: Option<String>,
    msg: Binary,
) -> Result<Response> {
    let msg: ExecuteMsg = from_binary(&msg)?;

    if matches!(msg, ExecuteMsg::Receive { .. }) {
        return Err(Error::Generic(
            "Recursive call to receive() is not allowed".to_string(),
        ));
    }

    info.sender = from;

    execute(deps, env, info, msg)
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response> {
    match (msg.id.into(), msg.result) {
        (CREATE_LB_PAIR_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(data) => {
                let lb_pair: lb_pair::LbPair = from_binary(&data)?;

                let data = CreateLbPairResponse { lb_pair };

                let mut msgs: Vec<CosmosMsg> = Vec::with_capacity(2);

                for token in [&data.lb_pair.token_x, &data.lb_pair.token_y] {
                    if token.is_custom_token() {
                        msgs.extend([
                            snip20::set_viewing_key_msg(
                                "hola".to_string(),
                                None,
                                1,
                                token.address().to_string(),
                                token.code_hash(),
                            )?,
                            snip20::register_receive_msg(
                                env.contract.code_hash.clone(),
                                None,
                                1,
                                token.address().to_string(),
                                token.code_hash(),
                            )?,
                        ])
                    }
                }

                Ok(Response::new()
                    .set_data(to_binary(&data)?)
                    .add_messages(msgs))
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

                let amounts_added = amounts_received.sub(amounts_left)?;

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

                Ok(Response::new().set_data(to_binary(&data)?))
            }
            None => Err(Error::ReplyDataMissing),
        },
        (BURN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(data) => {
                let lb_pair::BurnResponse {
                    amounts: amounts_burned,
                } = from_binary(&data)?;

                let EphemeralRemoveLiquidity {
                    amount_x_min,
                    amount_y_min,
                    is_wrong_order,
                } = EPHEMERAL_REMOVE_LIQUIDITY.load(deps.storage)?;

                let mut amount_x = 0u128;
                let mut amount_y = 0u128;

                for amount_burned in amounts_burned {
                    amount_x += amount_burned.decode_x();
                    amount_y += amount_burned.decode_y();
                }

                if is_wrong_order {
                    (amount_x, amount_y) = (amount_y, amount_x);
                }

                let amount_x = Uint128::from(amount_x);
                let amount_y = Uint128::from(amount_y);

                if amount_x < amount_x_min || amount_y < amount_y_min {
                    return Err(Error::AmountSlippageCaught {
                        amount_x_min,
                        amount_x,
                        amount_y_min,
                        amount_y,
                    });
                }

                let data = lb_router::RemoveLiquidityResponse { amount_x, amount_y };

                Ok(Response::new().set_data(to_binary(&data)?))
            }
            None => Err(Error::ReplyDataMissing),
        },
        (SWAP_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(data) => {
                let EphemeralSwap {
                    amount_in,
                    amount_out_min,
                    pairs,
                    versions,
                    token_path,
                    mut position,
                    mut token_next,
                    swap_for_y,
                    to,
                } = EPHEMERAL_SWAP.load(deps.storage)?;

                // let (amount_x_out, amount_y_out) = from_binary::<lb_pair::SwapResponse>(&data)?
                //     .amounts_out
                //     .decode();

                // TODO: see if this works
                let amounts_out: Bytes32 = data.to_vec().try_into().map_err(|v: Vec<u8>| {
                    Error::Generic(format!("Invalid length for Bytes32: got {} bytes", v.len()))
                })?;
                let (amount_x_out, amount_y_out) = amounts_out.decode();

                let amount_out = if swap_for_y {
                    Uint128::new(amount_y_out)
                } else {
                    Uint128::new(amount_x_out)
                };

                if amount_out_min > amount_out {
                    return Err(Error::InsufficientAmountOut {
                        amount_out_min,
                        amount_out,
                    });
                }

                position += 1;

                if position == token_path.len() as u32 {
                    let data = lb_router::SwapResponse { amount_out };

                    Ok(Response::new().set_data(to_binary(&data)?))
                } else {
                    token_next = token_path[position as usize].clone();

                    EPHEMERAL_SWAP.update(deps.storage, |mut data| -> StdResult<_> {
                        data.position = position;
                        data.token_next = token_next.clone();
                        Ok(data)
                    })?;

                    _swap_exact_tokens_for_tokens(
                        deps,
                        &env,
                        Response::new(),
                        amount_out,
                        pairs,
                        versions,
                        token_path,
                        position + 1,
                        token_next,
                        to,
                    )
                }
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
