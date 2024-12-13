#![allow(unused)]

use crate::{
    contract::{
        ensure, BURN_REPLY_ID, CREATE_LB_PAIR_REPLY_ID, MINT_REPLY_ID, ROUTER_KEY, SWAP_REPLY_ID,
    },
    helper::_get_lb_pair_information,
    prelude::*,
    state::{
        EphemeralAddLiquidity, EphemeralRemoveLiquidity, EPHEMERAL_ADD_LIQUIDITY,
        EPHEMERAL_REMOVE_LIQUIDITY, FACTORY,
    },
};
use cosmwasm_std::{
    to_binary, Addr, ContractInfo, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
    Uint128, Uint256, Uint64,
};
use lb_interfaces::{
    lb_factory,
    lb_pair::{self, LiquidityParameters},
    lb_router::{self, Path},
};
use lb_libraries::types::LiquidityConfiguration;
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
    let entropy = env.block.random.unwrap_or(to_binary(b"meh")?);

    let factory = FACTORY.load(deps.storage)?;
    let msg = lb_factory::ExecuteMsg::CreateLbPair {
        token_x,
        token_y,
        active_id,
        bin_step,
        viewing_key: ROUTER_KEY.to_string(),
        entropy: entropy.to_string(),
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
    ensure(env, liquidity_parameters.deadline.u64())?;

    let pair = _get_lb_pair_information(
        deps.as_ref(),
        liquidity_parameters.token_x.clone(),
        liquidity_parameters.token_y.clone(),
        liquidity_parameters.bin_step,
    )?;

    let lb_pair::TokenXResponse { token_x } =
        deps.querier.query_wasm_smart::<lb_pair::TokenXResponse>(
            pair.code_hash.clone(),
            pair.address.clone(),
            &lb_pair::QueryMsg::GetTokenX {},
        )?;

    if liquidity_parameters.token_x != token_x {
        return Err(Error::WrongTokenOrder);
    }

    // NOTE: Router requires token allowance before this function can be called.
    // TODO: Transfer tokens from sender to the pair contract.

    _add_liquidity(deps, liquidity_parameters, pair)
}

pub fn _add_liquidity(
    deps: DepsMut,
    liq: LiquidityParameters,
    pair: ContractInfo,
) -> Result<Response> {
    if liq.delta_ids.len() != liq.distribution_x.len()
        || liq.delta_ids.len() != liq.distribution_y.len()
    {
        return Err(Error::LengthsMismatch);
    }

    const UINT24_MAX: u32 = (1 << 24) - 1; // 2^24 - 1
    if liq.active_id_desired > UINT24_MAX || liq.id_slippage > UINT24_MAX {
        return Err(Error::IdDesiredOverflows {
            id_desired: liq.active_id_desired,
            id_slippage: liq.id_slippage,
        });
    }

    // TODO: encode these as Bytes32
    let mut liquidity_configs = vec![LiquidityConfiguration::default(); liq.delta_ids.len()];
    let mut deposit_ids = Vec::with_capacity(liq.delta_ids.len());

    let lb_pair::ActiveIdResponse { active_id } =
        deps.querier.query_wasm_smart::<lb_pair::ActiveIdResponse>(
            pair.code_hash.clone(),
            pair.address.clone(),
            &lb_pair::QueryMsg::GetActiveId {},
        )?;

    if liq.active_id_desired + liq.id_slippage < active_id
        || active_id + liq.id_slippage < liq.active_id_desired
    {
        return Err(Error::IdSlippageCaught {
            active_id_desired: liq.active_id_desired,
            id_slippage: liq.id_slippage,
            active_id,
        });
    }

    for (i, liquidity_config) in liquidity_configs.iter_mut().enumerate() {
        let id: i64 = active_id as i64 + liq.delta_ids[i];

        deposit_ids.push(id as u32);

        // TODO: encode these as Bytes32
        *liquidity_config = LiquidityConfiguration {
            distribution_x: liq.distribution_x[i].u64(),
            distribution_y: liq.distribution_y[i].u64(),
            id: id as u32,
        };
    }

    EPHEMERAL_ADD_LIQUIDITY.save(
        deps.storage,
        &EphemeralAddLiquidity {
            amount_x_min: liq.amount_x_min,
            amount_y_min: liq.amount_y_min,
            deposit_ids,
        },
    )?;

    let msg = lb_pair::ExecuteMsg::Mint {
        to: liq.to,
        liquidity_configs,
        refund_to: liq.refund_to,
    }
    .to_cosmos_msg(&pair, vec![])?;

    let response = Response::new().add_submessage(SubMsg::reply_on_success(msg, MINT_REPLY_ID));

    Ok(response)
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
    mut amount_x_min: Uint128,
    mut amount_y_min: Uint128,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
    to: String,
    deadline: Uint64,
) -> Result<Response> {
    // ILBPair _LBPair = ILBPair(_getLBPairInformation(tokenX, tokenY, binStep, Version.V2_2));
    // bool isWrongOrder = tokenX != _LBPair.getTokenX();
    //
    // if (isWrongOrder) (amountXMin, amountYMin) = (amountYMin, amountXMin);
    //
    // (amountX, amountY) = _removeLiquidity(_LBPair, amountXMin, amountYMin, ids, amounts, to);
    //
    // if (isWrongOrder) (amountX, amountY) = (amountY, amountX);

    // perform checks
    // query for the pair contract
    // do internal _remove_liquidity
    // save some info in ephemeral storage
    // check for errors in the reply

    let to = deps.api.addr_validate(&to)?;

    let pair = _get_lb_pair_information(
        deps.as_ref(),
        token_x.clone().into(),
        token_y.clone().into(),
        bin_step.clone(),
    )?;

    let lb_pair::TokenXResponse {
        token_x: lb_pair_token_x,
    } = deps.querier.query_wasm_smart::<lb_pair::TokenXResponse>(
        pair.code_hash.clone(),
        pair.address.clone(),
        &lb_pair::QueryMsg::GetTokenX {},
    )?;

    let is_wrong_order = TokenType::from(token_x) != lb_pair_token_x;

    if is_wrong_order {
        (amount_x_min, amount_y_min) = (amount_y_min, amount_x_min)
    }

    EPHEMERAL_REMOVE_LIQUIDITY.save(
        deps.storage,
        &EphemeralRemoveLiquidity {
            amount_x_min,
            amount_y_min,
            is_wrong_order,
        },
    );

    _remove_liquidity(
        deps,
        env,
        info,
        pair,
        amount_x_min,
        amount_y_min,
        ids,
        amounts,
        to,
    )
}

pub fn _remove_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pair: ContractInfo,
    amount_x_min: Uint128,
    amount_y_min: Uint128,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
    to: Addr,
) -> Result<Response> {
    // call the `burn` message in lb-pair
    // handle the `amounts_burned` in the submsg reply

    // TODO: this is annoying... we already validated these Addr, but we need to change them
    // back to Strings.
    let msg = lb_pair::ExecuteMsg::Burn {
        from: info.sender.to_string(),
        to: to.to_string(),
        ids,
        amounts_to_burn: amounts,
    }
    .to_cosmos_msg(&pair, vec![])?;

    let response = Response::new().add_submessage(SubMsg::reply_on_success(msg, BURN_REPLY_ID));

    Ok(response)
}

// TODO: rewrite this old function from lb_pair
// fn remove_liquidity(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     _to: Addr,
//     amount_x_min: Uint128,
//     amount_y_min: Uint128,
//     ids: Vec<u32>,
//     amounts: Vec<Uint256>,
// ) -> Result<(Uint128, Uint128, Response)> {
//     let (amounts_burned, response) = burn(deps, env, info, ids, amounts)?;
//     let mut amount_x: Uint128 = Uint128::zero();
//     let mut amount_y: Uint128 = Uint128::zero();
//     for amount_burned in amounts_burned {
//         amount_x += Uint128::from(amount_burned.decode_x());
//         amount_y += Uint128::from(amount_burned.decode_y());
//     }
//
//     if amount_x < amount_x_min || amount_y < amount_y_min {
//         return Err(Error::AmountSlippageCaught {
//             amount_x_min,
//             amount_x,
//             amount_y_min,
//             amount_y,
//         });
//     }
//
//     Ok((amount_x, amount_y, response))
// }

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
    amount_out: Uint256,
    amount_in_max: Uint256,
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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: ContractInfo, // must be a snip20 token
    to: String,
    amount: Uint128,
) -> Result<Response> {
    todo!()
}

pub fn sweep_lb_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: ContractInfo, // must be an LbToken
    to: String,
    ids: Vec<u32>,
    amounts: Vec<Uint128>,
) -> Result<Response> {
    todo!()
}
