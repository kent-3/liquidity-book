// #![allow(unused)]

use crate::{
    contract::{ensure, BURN_REPLY_ID, CREATE_LB_PAIR_REPLY_ID, MINT_REPLY_ID, ROUTER_KEY},
    helper::*,
    prelude::*,
    state::*,
};
use cosmwasm_std::{
    to_binary, Addr, ContractInfo, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128,
    Uint256, Uint64,
};
use lb_interfaces::{
    lb_factory,
    lb_pair::{self, ILbPair, LiquidityParameters},
    lb_router::{Path, Version},
};
use lb_libraries::types::LiquidityConfiguration;
use shade_protocol::{swap::core::TokenType, utils::ExecuteCallback};

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
    ensure(&env, liquidity_parameters.deadline.u64())?;

    // TODO: the LiquidityParameters token inputs should really be ContractInfo instead
    let token_x = liquidity_parameters
        .token_x
        .clone()
        .into_contract_info()
        .unwrap();
    let token_y = liquidity_parameters
        .token_y
        .clone()
        .into_contract_info()
        .unwrap();

    let pair = ILbPair(_get_lb_pair_information(
        deps.as_ref(),
        token_x,
        token_y,
        liquidity_parameters.bin_step,
        Version::V2_2,
    )?);

    let token_x = pair.get_token_x(deps.querier)?;

    if liquidity_parameters.token_x != token_x.into() {
        return Err(Error::WrongTokenOrder);
    }

    // NOTE: Router requires token allowance before this function can be called.
    // Could increasing allowances be done via submessages? I don't think so, because the message
    // sender would be this contract, not the user.

    // TODO: Transfer tokens from sender to the pair contract.
    let transfer_x_msg = secret_toolkit::snip20::transfer_msg(
        pair.0.address.to_string(),
        liquidity_parameters.amount_x,
        None,
        None,
        32,
        pair.0.code_hash.clone(),
        pair.0.address.to_string(),
    )?;
    let transfer_y_msg = secret_toolkit::snip20::transfer_msg(
        pair.0.address.to_string(),
        liquidity_parameters.amount_y,
        None,
        None,
        32,
        pair.0.code_hash.clone(),
        pair.0.address.to_string(),
    )?;

    let response = Response::new().add_messages(vec![transfer_x_msg, transfer_y_msg]);

    _add_liquidity(deps, response, liquidity_parameters, pair)
}

pub fn _add_liquidity(
    deps: DepsMut,
    response: Response,
    liq: LiquidityParameters,
    pair: ILbPair,
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

    let active_id = pair.get_active_id(deps.querier)?;

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

    // TODO: add ExecuteMsg methods to ILbPair?

    let lb_pair_mint_msg = SubMsg::reply_on_success(
        lb_pair::ExecuteMsg::Mint {
            to: liq.to,
            liquidity_configs,
            refund_to: liq.refund_to,
        }
        .to_cosmos_msg(&pair.0, vec![])?,
        MINT_REPLY_ID,
    );

    Ok(response.add_submessage(lb_pair_mint_msg))
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
        Version::V2_2,
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
    ensure(&env, deadline.u64())?;

    let pairs = _get_pairs(
        deps.as_ref(),
        path.pair_bin_steps,
        path.versions.clone(),
        path.token_path.clone(),
    )?;

    // TODO: transfer received tokens to the lb_pair contract

    _swap_exact_tokens_for_tokens(
        deps,
        &env,
        info,
        amount_in,
        pairs,
        path.versions,
        path.token_path,
        to,
    )

    // TODO: store amount_out_min in ephemeral storage

    // TODO: add this check in the submsg reply
    //     if (amountOutMin > amountOut) revert LBRouter__InsufficientAmountOut(amountOutMin, amountOut);
}

pub fn _swap_exact_tokens_for_tokens(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    amount_in: Uint256,
    pairs: Vec<ContractInfo>,
    versions: Vec<Version>,
    token_path: Vec<ContractInfo>,
    to: String,
) -> Result<Response> {
    todo!()
}

// function _swapExactTokensForTokens(
//     uint256 amountIn,
//     address[] memory pairs,
//     Version[] memory versions,
//     IERC20[] memory tokenPath,
//     address to
// ) private returns (uint256 amountOut) {
//     IERC20 token;
//     Version version;
//     address recipient;
//     address pair;
//
//     IERC20 tokenNext = tokenPath[0];
//     amountOut = amountIn;
//
//     unchecked {
//         for (uint256 i; i < pairs.length; ++i) {
//             pair = pairs[i];
//             version = versions[i];
//
//             token = tokenNext;
//             tokenNext = tokenPath[i + 1];
//
//             recipient = i + 1 == pairs.length ? to : pairs[i + 1];
//
//             if (version == Version.V1) {
//                 (uint256 reserve0, uint256 reserve1,) = IJoePair(pair).getReserves();
//
//                 if (token < tokenNext) {
//                     amountOut = amountOut.getAmountOut(reserve0, reserve1);
//                     IJoePair(pair).swap(0, amountOut, recipient, "");
//                 } else {
//                     amountOut = amountOut.getAmountOut(reserve1, reserve0);
//                     IJoePair(pair).swap(amountOut, 0, recipient, "");
//                 }
//             } else if (version == Version.V2) {
//                 bool swapForY = tokenNext == ILBLegacyPair(pair).tokenY();
//
//                 (uint256 amountXOut, uint256 amountYOut) = ILBLegacyPair(pair).swap(swapForY, recipient);
//
//                 if (swapForY) amountOut = amountYOut;
//                 else amountOut = amountXOut;
//             } else {
//                 bool swapForY = tokenNext == ILBPair(pair).getTokenY();
//
//                 (uint256 amountXOut, uint256 amountYOut) = ILBPair(pair).swap(swapForY, recipient).decode();
//
//                 if (swapForY) amountOut = amountYOut;
//                 else amountOut = amountXOut;
//             }
//         }
//     }
// }

/**
 * @notice Swaps exact tokens for tokens while performing safety checks
 * @param amountIn The amount of token to send
 * @param amountOutMin The min amount of token to receive
 * @param path The path of the swap
 * @param to The address of the recipient
 * @param deadline The deadline of the tx
 * @return amountOut Output amount of the swap
 */
// function swapExactTokensForTokens(
//     uint256 amountIn,
//     uint256 amountOutMin,
//     Path memory path,
//     address to,
//     uint256 deadline
// ) external override ensure(deadline) verifyPathValidity(path) returns (uint256 amountOut) {
//     address[] memory pairs = _getPairs(path.pairBinSteps, path.versions, path.tokenPath);
//
//     _safeTransferFrom(path.tokenPath[0], msg.sender, pairs[0], amountIn);
//
//     amountOut = _swapExactTokensForTokens(amountIn, pairs, path.versions, path.tokenPath, to);
//
//     if (amountOutMin > amountOut) revert LBRouter__InsufficientAmountOut(amountOutMin, amountOut);
// }

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
