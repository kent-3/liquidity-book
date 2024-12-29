// #![allow(unused)]

// TODO: function doc comments

use crate::{contract::*, helper::*, prelude::*, query::query_swap_in, state::*};
use cosmwasm_std::{
    to_binary, Addr, ContractInfo, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
    Uint128, Uint256, Uint64,
};
use liquidity_book::{
    interfaces::{
        lb_factory,
        lb_pair::{self, ILbPair, LiquidityParameters, SwapInResponse},
        lb_router::{Path, Version},
    },
    libraries::LiquidityConfigurations,
};
use shade_protocol::{swap::core::TokenType, utils::ExecuteCallback};

pub fn create_lb_pair(
    deps: DepsMut,
    env: Env,
    token_x: TokenType,
    token_y: TokenType,
    active_id: u32,
    bin_step: u16,
) -> Result<Response> {
    let entropy = env.block.random.unwrap_or(to_binary(b"meh")?); // TODO:

    let factory = FACTORY.load(deps.storage)?;

    // TODO: decide which version is best

    let msg = factory.create_lb_pair(
        token_x,
        token_y,
        active_id,
        bin_step,
        PUBLIC_VIEWING_KEY.to_string(),
        entropy.to_string(),
    )?;

    // let msg = lb_factory::ExecuteMsg::CreateLbPair {
    //     token_x,
    //     token_y,
    //     active_id,
    //     bin_step,
    //     viewing_key: PUBLIC_VIEWING_KEY.to_string(),
    //     entropy: entropy.to_string(),
    // }
    // .to_cosmos_msg(&factory.0, vec![])?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, CREATE_LB_PAIR_REPLY_ID)))
}

pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    liquidity_parameters: LiquidityParameters,
) -> Result<Response> {
    // TODO: original has this check happening in _add_liquidity. should I move there also?
    ensure(&env, liquidity_parameters.deadline.u64())?;

    // let token_x = liquidity_parameters
    //     .token_x
    //     .clone()
    //     .into_contract_info()
    //     .unwrap();
    // let token_y = liquidity_parameters
    //     .token_y
    //     .clone()
    //     .into_contract_info()
    //     .unwrap();

    let lb_pair = ILbPair(_get_lb_pair_information(
        deps.as_ref(),
        liquidity_parameters.token_x.clone(),
        liquidity_parameters.token_y.clone(),
        liquidity_parameters.bin_step,
        Version::V2_2,
    )?);

    if liquidity_parameters.token_x != lb_pair.get_token_x(deps.querier)?.into() {
        return Err(Error::WrongTokenOrder);
    }

    // NOTE: Router requires token allowance before this function can be called.
    // Could increasing allowances be done via submessages? I don't think so, because the message
    // sender would be this contract, not the user.

    // TODO: handle TokenType::Native

    // Transfer tokens from sender to the pair contract.
    let transfer_x_msg = secret_toolkit::snip20::transfer_from_msg(
        info.sender.to_string(),
        lb_pair.0.address.to_string(),
        liquidity_parameters.amount_x,
        None,
        None,
        32,
        liquidity_parameters.token_x.code_hash().clone(),
        liquidity_parameters.token_x.address().to_string(),
    )?;
    let transfer_y_msg = secret_toolkit::snip20::transfer_from_msg(
        info.sender.to_string(),
        lb_pair.0.address.to_string(),
        liquidity_parameters.amount_y,
        None,
        None,
        32,
        liquidity_parameters.token_y.code_hash().clone(),
        liquidity_parameters.token_y.address().to_string(),
    )?;

    let response = Response::new().add_messages(vec![transfer_x_msg, transfer_y_msg]);

    _add_liquidity(deps, response, liquidity_parameters, lb_pair)
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

    let mut liquidity_configs = vec![LiquidityConfigurations::default(); liq.delta_ids.len()];
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

        *liquidity_config = LiquidityConfigurations::encode_params(
            liq.distribution_x[i].u64(),
            liq.distribution_y[i].u64(),
            id as u32,
        );
    }

    // NOTE: See reply in contract.rs for continuation of this function.

    EPHEMERAL_ADD_LIQUIDITY.save(
        deps.storage,
        &EphemeralAddLiquidity {
            amount_x_min: liq.amount_x_min,
            amount_y_min: liq.amount_y_min,
            deposit_ids,
        },
    )?;

    let lb_pair_mint_msg = pair.mint(liq.to, liquidity_configs, liq.refund_to)?;
    let response =
        response.add_submessage(SubMsg::reply_on_success(lb_pair_mint_msg, MINT_REPLY_ID));

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
    ensure(&env, deadline.u64())?;
    let to = deps.api.addr_validate(&to)?;

    let lb_pair = ILbPair(_get_lb_pair_information(
        deps.as_ref(),
        token_x.clone().into(),
        token_y.clone().into(),
        bin_step.clone(),
        Version::V2_2,
    )?);
    let is_wrong_order = TokenType::from(token_x) != lb_pair.get_token_x(deps.querier)?;

    if is_wrong_order {
        (amount_x_min, amount_y_min) = (amount_y_min, amount_x_min)
    }

    // NOTE: See reply in contract.rs for continuation of this function.

    EPHEMERAL_REMOVE_LIQUIDITY.save(
        deps.storage,
        &EphemeralRemoveLiquidity {
            amount_x_min,
            amount_y_min,
            is_wrong_order,
        },
    )?;

    let lb_pair_burn_msg = lb_pair.burn(info.sender.to_string(), to.to_string(), ids, amounts)?;
    let response =
        Response::new().add_submessage(SubMsg::reply_on_success(lb_pair_burn_msg, BURN_REPLY_ID));

    Ok(response)
}

pub fn remove_liquidity_native() {
    unimplemented!()
}

/**
 * @notice Swaps exact tokens for tokens while performing safety checks
 * @param amountIn The amount of token to send
 * @param amountOutMin The min amount of token to receive
 * @param path The path of the swap
 * @param to The address of the recipient
 * @param deadline The deadline of the tx
 * @return amountOut Output amount of the swap
 */
pub fn swap_exact_tokens_for_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount_in: Uint128,
    amount_out_min: Uint128,
    path: Path,
    to: String,
    deadline: Uint64,
) -> Result<Response> {
    ensure(&env, deadline.u64())?;
    verify_path_validity(&path)?;

    let to = deps.api.addr_validate(&to)?;

    let pairs = _get_pairs(
        deps.as_ref(),
        path.pair_bin_steps,
        path.versions.clone(),
        path.token_path.clone(),
    )?;

    // Transfer tokens from this router contract to the pair contract.
    let transfer_msg = secret_toolkit::snip20::transfer_msg(
        pairs[0].address.to_string(),
        amount_in,
        None,
        None,
        32,
        path.token_path[0].code_hash().clone(),
        path.token_path[0].address().to_string(),
    )?;

    let response = Response::new().add_message(transfer_msg);

    let token_next = path.token_path[0].clone();

    EPHEMERAL_SWAP.save(
        deps.storage,
        &EphemeralSwap {
            amount_in,
            amount_out_min,
            pairs: pairs.clone(),
            versions: path.versions.clone(),
            token_path: path.token_path.clone(),
            position: 0,
            token_next: token_next.clone(),
            swap_for_y: false,
            to: to.clone(),
        },
    )?;

    _swap_exact_tokens_for_tokens(
        deps,
        &env,
        response,
        amount_in,
        pairs,
        path.versions,
        path.token_path,
        0,
        token_next,
        to,
    )
}

// NOTE: `amount_out` and `token` aren't used in LB, but might be needed to support other swaps in
// the future.

pub fn _swap_exact_tokens_for_tokens(
    deps: DepsMut,
    _env: &Env,
    response: Response,
    _amount_out: Uint128,
    pairs: Vec<ILbPair>,
    versions: Vec<Version>,
    token_path: Vec<TokenType>,
    position: u32,
    token_next: TokenType,
    to: Addr,
) -> Result<Response> {
    let i = position as usize;

    let pair = pairs[i].clone();
    let version = versions[i].clone();

    let _token = token_next;
    let token_next = token_path[i + 1].clone();

    let recipient = if i + 1 == pairs.len() {
        to
    } else {
        // we are sending the tokens obtained in the swap directly to the next lb_pair contract!
        pairs[i + 1].address.clone()
    };

    if version == Version::V1 {
        unimplemented!()
    } else if version == Version::V2 {
        unimplemented!()
    } else {
        let swap_for_y = token_next == pair.get_token_y(deps.querier)?;

        // TODO: annoying
        EPHEMERAL_SWAP.update(deps.storage, |mut data| -> StdResult<_> {
            data.swap_for_y = swap_for_y;
            Ok(data)
        })?;

        let lb_pair_swap_msg = pair.swap(swap_for_y, recipient.to_string())?;
        let response =
            response.add_submessage(SubMsg::reply_on_success(lb_pair_swap_msg, SWAP_REPLY_ID));

        Ok(response)
    }
}

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

pub fn swap_exact_tokens_for_native() {
    unimplemented!()
}

pub fn swap_exact_native_for_tokens() {
    unimplemented!()
}

/**
 * @notice Swaps tokens for exact tokens while performing safety checks
 * @param amountOut The amount of token to receive
 * @param amountInMax The max amount of token to send
 * @param path The path of the swap
 * @param to The address of the recipient
 * @param deadline The deadline of the tx
 * @return amountsIn Input amounts of the swap
 */
pub fn swap_tokens_for_exact_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount_out: Uint128,
    amount_in_max: Uint128,
    path: Path,
    to: String,
    deadline: Uint64,
) -> Result<Response> {
    ensure(&env, deadline.u64())?;
    verify_path_validity(&path)?;

    let to = deps.api.addr_validate(&to)?;

    let pairs = _get_pairs(
        deps.as_ref(),
        path.pair_bin_steps,
        path.versions.clone(),
        path.token_path.clone(),
    )?;

    let amounts_in = _get_amounts_in(
        deps.as_ref(),
        path.versions.clone(),
        pairs.clone(),
        path.token_path.clone(),
        amount_out,
    )?;

    if amounts_in[0] > amount_in_max {
        return Err(Error::MaxAmountInExceeded {
            amount_in_max,
            amount_in: amounts_in[0],
        });
    }

    // Transfer tokens from this router contract to the pair contract.
    let transfer_msg = secret_toolkit::snip20::transfer_msg(
        pairs[0].address.to_string(),
        amounts_in[0],
        None,
        None,
        32,
        path.token_path[0].code_hash().clone(),
        path.token_path[0].address().to_string(),
    )?;

    let response = Response::new().add_message(transfer_msg);

    // TODO: this is handled in the reply.
    //         uint256 _amountOutReal = _swapTokensForExactTokens(pairs, path.versions, path.tokenPath, amountsIn, to);
    //
    //         if (_amountOutReal < amountOut) revert LBRouter__InsufficientAmountOut(amountOut, _amountOutReal);

    let token_next = path.token_path[0].clone();

    EPHEMERAL_SWAP.save(
        deps.storage,
        &EphemeralSwap {
            // TODO: rename the fields to be more generic. used in both swap directions.
            // or make separate ones for the different swap types. that's probably better
            amount_in: amount_out,         // amount_out
            amount_out_min: amount_in_max, // amount_in_max
            pairs: pairs.clone(),
            versions: path.versions.clone(),
            token_path: path.token_path.clone(),
            position: 0,
            token_next: token_next.clone(),
            swap_for_y: false,
            to: to.clone(),
        },
    )?;

    _swap_tokens_for_exact_tokens(
        deps,
        &env,
        response,
        pairs,
        path.versions,
        path.token_path,
        amounts_in,
        0,
        token_next,
        to,
    )
}

// NOTE: `amounts_in` and `token` aren't used in LB, but might be needed to support other swaps in
// the future.

// TODO: double check this closely... It might be the same as swap_exact_tokens_for_tokens
pub fn _swap_tokens_for_exact_tokens(
    deps: DepsMut,
    _env: &Env,
    response: Response,
    pairs: Vec<ILbPair>,
    versions: Vec<Version>,
    token_path: Vec<TokenType>,
    amounts_in: Vec<Uint128>,
    position: u32,
    token_next: TokenType,
    to: Addr,
) -> Result<Response> {
    let i = position as usize;

    let pair = pairs[i].clone();
    let version = versions[i].clone();

    let _token = token_next;
    let token_next = token_path[i + 1].clone();

    let recipient = if i + 1 == pairs.len() {
        to
    } else {
        // we are sending the tokens obtained in the swap directly to the next lb_pair contract!
        pairs[i + 1].address.clone()
    };

    match version {
        Version::V1 => unimplemented!(),
        Version::V2 => unimplemented!(),
        _ => {
            let swap_for_y = token_next == pair.get_token_y(deps.querier)?;

            // TODO: annoying
            EPHEMERAL_SWAP.update(deps.storage, |mut data| -> StdResult<_> {
                data.swap_for_y = swap_for_y;
                Ok(data)
            })?;

            let lb_pair_swap_msg = pair.swap(swap_for_y, recipient.to_string())?;
            let response =
                response.add_submessage(SubMsg::reply_on_success(lb_pair_swap_msg, SWAP_REPLY_ID));

            Ok(response)
        }
    }
}

pub fn swap_tokens_for_exact_native() {
    unimplemented!()
}

/**
 * @notice Helper function to return the amounts in
 * @param versions The list of versions (V1, V2, V2_1 or V2_2)
 * @param pairs The list of pairs
 * @param tokenPath The swap path
 * @param amountOut The amount out
 * @return amountsIn The list of amounts in
 */
pub fn _get_amounts_in(
    deps: Deps,
    versions: Vec<Version>,
    pairs: Vec<ILbPair>,
    token_path: Vec<TokenType>,
    amount_out: Uint128,
) -> Result<Vec<Uint128>> {
    let mut amounts_in: Vec<Uint128> = vec![Uint128::zero(); token_path.len()];
    // Avoid doing -1, as `pairs.length == pairBinSteps.length-1`
    amounts_in[pairs.len()] = amount_out;

    for i in (1..=pairs.len()).rev() {
        let token = token_path[i - 1].clone();
        let version = versions[i - 1].clone();
        let pair = pairs[i - 1].clone();

        amounts_in[i - 1] = match version {
            Version::V1 => unimplemented!(),
            Version::V2 => unimplemented!(),
            _ => {
                query_swap_in(
                    deps,
                    pair.0.clone(),
                    amounts_in[i],
                    pair.get_token_x(deps.querier)? == token,
                )?
                .amount_in
            }
        }
    }

    Ok(amounts_in)
}

// function swapTokensForExactTokens(
//     uint256 amountOut,
//     uint256 amountInMax,
//     Path memory path,
//     address to,
//     uint256 deadline
// ) external override ensure(deadline) verifyPathValidity(path) returns (uint256[] memory amountsIn) {
//     address[] memory pairs = _getPairs(path.pairBinSteps, path.versions, path.tokenPath);
//
//     {
//         amountsIn = _getAmountsIn(path.versions, pairs, path.tokenPath, amountOut);
//
//         if (amountsIn[0] > amountInMax) revert LBRouter__MaxAmountInExceeded(amountInMax, amountsIn[0]);
//
//         _safeTransferFrom(path.tokenPath[0], msg.sender, pairs[0], amountsIn[0]);
//
//         uint256 _amountOutReal = _swapTokensForExactTokens(pairs, path.versions, path.tokenPath, amountsIn, to);
//
//         if (_amountOutReal < amountOut) revert LBRouter__InsufficientAmountOut(amountOut, _amountOutReal);
//     }
// }

/**
 * @notice Unstuck tokens that are sent to this contract by mistake
 * @dev Only callable by the factory owner
 * @param token The address of the token
 * @param to The address of the user to send back the tokens
 * @param amount The amount to send
 */
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

/**
 * @notice Unstuck LBTokens that are sent to this contract by mistake
 * @dev Only callable by the factory owner
 * @param lbToken The address of the LBToken
 * @param to The address of the user to send back the tokens
 * @param ids The list of token ids
 * @param amounts The list of amounts to send
 */
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

// function sweep(IERC20 token, address to, uint256 amount) external override onlyFactoryOwner {
//     if (address(token) == address(0)) {
//         amount = amount == type(uint256).max ? address(this).balance : amount;
//
//         _safeTransferNative(to, amount);
//     } else {
//         amount = amount == type(uint256).max ? token.balanceOf(address(this)) : amount;
//
//         token.safeTransfer(to, amount);
//     }
// }
//
// function sweepLBToken(ILBToken lbToken, address to, uint256[] calldata ids, uint256[] calldata amounts)
//     external
//     override
//     onlyFactoryOwner
// {
//     lbToken.batchTransferFrom(address(this), to, ids, amounts);
// }
//
// /**
//  * @notice Helper function to add liquidity
//  * @param liq The liquidity parameter
//  * @param pair LBPair where liquidity is deposited
//  * @return amountXAdded Amount of token X added
//  * @return amountYAdded Amount of token Y added
//  * @return amountXLeft Amount of token X left
//  * @return amountYLeft Amount of token Y left
//  * @return depositIds The list of deposit ids
//  * @return liquidityMinted The list of liquidity minted
//  */
// function _addLiquidity(LiquidityParameters calldata liq, ILBPair pair)
//     private
//     ensure(liq.deadline)
//     returns (
//         uint256 amountXAdded,
//         uint256 amountYAdded,
//         uint256 amountXLeft,
//         uint256 amountYLeft,
//         uint256[] memory depositIds,
//         uint256[] memory liquidityMinted
//     )
// {
//     unchecked {
//         if (liq.deltaIds.length != liq.distributionX.length || liq.deltaIds.length != liq.distributionY.length) {
//             revert LBRouter__LengthsMismatch();
//         }
//
//         if (liq.activeIdDesired > type(uint24).max || liq.idSlippage > type(uint24).max) {
//             revert LBRouter__IdDesiredOverflows(liq.activeIdDesired, liq.idSlippage);
//         }
//
//         bytes32[] memory liquidityConfigs = new bytes32[](liq.deltaIds.length);
//         depositIds = new uint256[](liq.deltaIds.length);
//         {
//             uint256 _activeId = pair.getActiveId();
//             if (
//                 liq.activeIdDesired + liq.idSlippage < _activeId || _activeId + liq.idSlippage < liq.activeIdDesired
//             ) {
//                 revert LBRouter__IdSlippageCaught(liq.activeIdDesired, liq.idSlippage, _activeId);
//             }
//
//             for (uint256 i; i < liquidityConfigs.length; ++i) {
//                 int256 _id = int256(_activeId) + liq.deltaIds[i];
//
//                 if (_id < 0 || uint256(_id) > type(uint24).max) revert LBRouter__IdOverflows(_id);
//                 depositIds[i] = uint256(_id);
//                 liquidityConfigs[i] = LiquidityConfigurations.encodeParams(
//                     uint64(liq.distributionX[i]), uint64(liq.distributionY[i]), uint24(uint256(_id))
//                 );
//             }
//         }
//
//         bytes32 amountsReceived;
//         bytes32 amountsLeft;
//         (amountsReceived, amountsLeft, liquidityMinted) = pair.mint(liq.to, liquidityConfigs, liq.refundTo);
//
//         bytes32 amountsAdded = amountsReceived.sub(amountsLeft);
//
//         amountXAdded = amountsAdded.decodeX();
//         amountYAdded = amountsAdded.decodeY();
//
//         if (amountXAdded < liq.amountXMin || amountYAdded < liq.amountYMin) {
//             revert LBRouter__AmountSlippageCaught(liq.amountXMin, amountXAdded, liq.amountYMin, amountYAdded);
//         }
//
//         amountXLeft = amountsLeft.decodeX();
//         amountYLeft = amountsLeft.decodeY();
//     }
// }
