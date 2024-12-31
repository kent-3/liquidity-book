// #![allow(unused)]

// TODO: function doc comments

use crate::{contract::*, prelude::*, query::query_swap_in, state::*};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, ContractInfo, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, SubMsg, Uint128, Uint256, Uint64,
};
use liquidity_book::{
    interfaces::{
        lb_pair::{ILbPair, LiquidityParameters},
        lb_router::{Path, Version},
    },
    libraries::{math::u24::U24, LiquidityConfigurations},
};
use secret_toolkit::snip20;
use shade_protocol::swap::core::TokenType;

/// Create a liquidity bin LBPair for token_x and token_y using the factory.
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

    let msg = factory.create_lb_pair(
        token_x,
        token_y,
        active_id,
        bin_step,
        PUBLIC_VIEWING_KEY.to_string(),
        entropy.to_string(),
    )?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(msg, CREATE_LB_PAIR_REPLY_ID)))
}

/// Add liquidity while performing safety checks.
///
/// This function is compliant with fee on transfer tokens.
pub fn add_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    liquidity_parameters: LiquidityParameters,
) -> Result<Response> {
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

    // Transfer tokens from sender to the pair contract.
    let transfer_from_x = _safe_transfer_from(
        &liquidity_parameters.token_x,
        &info.sender,
        &lb_pair.0.address,
        liquidity_parameters.amount_x,
    )?;
    let transfer_from_y = _safe_transfer_from(
        &liquidity_parameters.token_y,
        &info.sender,
        &lb_pair.0.address,
        liquidity_parameters.amount_y,
    )?;

    let response = [transfer_from_x, transfer_from_y]
        .into_iter()
        .filter_map(|msg| msg)
        .fold(Response::new(), |resp, msg| resp.add_message(msg));

    _add_liquidity(deps, env, response, liquidity_parameters, lb_pair)
}

/// Add liquidity with NATIVE while performing safety checks.
///
/// This function is compliant with fee on transfer tokens.
pub fn add_liquidity_native(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _liquidity_parameters: LiquidityParameters,
) -> Result<Response> {
    unimplemented!()
}

/// Remove liquidity while performing safety checks.
///
/// This function is compliant with fee on transfer tokens.
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

    EPHEMERAL_REMOVE_LIQUIDITY.save(
        deps.storage,
        &EphemeralRemoveLiquidity {
            amount_x_min,
            amount_y_min,
            is_wrong_order,
        },
    )?;

    _remove_liquidity(
        deps,
        env,
        info,
        lb_pair,
        amount_x_min,
        amount_y_min,
        ids,
        amounts,
        to,
    )
}

/// Remove NATIVE liquidity while performing safety checks.
///
/// This function is **NOT** compliant with fee on transfer tokens.
/// This is wanted as it would make users pays the fee on transfer twice,
/// use the `removeLiquidity` function to remove liquidity with fee on transfer tokens.
pub fn remove_liquidity_native(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _token: TokenType,
    _bin_step: u16,
    mut _amount_token_min: Uint128,
    mut _amount_native_min: Uint128,
    _ids: Vec<u32>,
    _amounts: Vec<Uint256>,
    _to: String,
    _deadline: Uint64,
) {
    unimplemented!()

    // TODO: I think the original version treats each NATIVE token as a wrapped ERC20 and just handles
    // the `deposit` and `redeem` conversions. But cosmos SDK chains are different in that they allow
    // multiple "native" tokens on each chain, and nobody wants/expects to use a wrapped version for
    // them. I don't think we want to be creating new cw20 or snip20 contracts for each one.
    // Fortunately, Secret doesn't use those, but still something to think about...
}

/// Swaps exact tokens for tokens while performing safety checks.
pub fn swap_exact_tokens_for_tokens(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
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

    // Transfer tokens from this router contract to the pair contract. They should have been
    // received via the SNIP20 receiver interface.
    let transfer_msg = _safe_transfer(&path.token_path[0], &pairs[0].address, amount_in)?
        .expect("the amounts_in should be non-zero");

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

/// Swaps exact tokens for NATIVE while performing safety checks.
pub fn swap_exact_tokens_for_native() {
    unimplemented!()
}

/// Swaps exact NATIVE for tokens while performing safety checks.
pub fn swap_exact_native_for_tokens() {
    unimplemented!()
}

/// Swaps tokens for exact tokens while performing safety checks.
pub fn swap_tokens_for_exact_tokens(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
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

    // Transfer tokens from this router contract to the pair contract. They should have been
    // received via the SNIP20 receiver interface.
    let transfer_msg = _safe_transfer(&path.token_path[0], &pairs[0].address, amounts_in[0])?
        .expect("the amounts_in should be non-zero");

    let response = Response::new().add_message(transfer_msg);

    let token_next = path.token_path[0].clone();

    EPHEMERAL_SWAP_FOR_EXACT.save(
        deps.storage,
        &EphemeralSwapForExact {
            amount_out,
            pairs: pairs.clone(),
            versions: path.versions.clone(),
            token_path: path.token_path.clone(),
            amounts_in: amounts_in.clone(),
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

pub fn swap_tokens_for_exact_native() {
    unimplemented!()
}

/// Unstuck tokens that are sent to this contract by mistake.
///
/// Only callable by the factory owner.
#[allow(unused)]
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

/// Unstuck LBTokens that are sent to this contract by mistake.
///
/// Only callable by the factory owner.
#[allow(unused)]
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

/// Helper function to add liquidity.
pub fn _add_liquidity(
    deps: DepsMut,
    env: Env,
    response: Response,
    liq: LiquidityParameters,
    pair: ILbPair,
) -> Result<Response> {
    ensure(&env, liq.deadline.u64())?;

    if liq.delta_ids.len() != liq.distribution_x.len()
        || liq.delta_ids.len() != liq.distribution_y.len()
    {
        return Err(Error::LengthsMismatch);
    }

    if liq.active_id_desired > U24::MAX || liq.id_slippage > U24::MAX {
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

/// Helper function to return the amounts in.
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

/// Helper function to remove liquidity.
pub fn _remove_liquidity(
    _deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pair: ILbPair,
    _amount_x_min: Uint128,
    _amount_y_min: Uint128,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
    to: Addr,
) -> Result<Response> {
    let burn_msg = pair.burn(info.sender.to_string(), to.to_string(), ids, amounts)?;

    // NOTE: See reply in contract.rs for continuation of this function.

    Ok(Response::new().add_submessage(SubMsg::reply_on_success(burn_msg, BURN_REPLY_ID)))
}

// NOTE: `amount_out` and `token` aren't used in LB, but might be needed to support other swap
// types in the future.

/// Helper function to swap exact tokens for tokens.
pub fn _swap_exact_tokens_for_tokens(
    deps: DepsMut,
    _env: &Env,
    response: Response,
    _amount_out: Uint128, // only used in Version::V1
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

// NOTE: `amounts_in` and `token` aren't used in LB, but might be needed to support other swaps in
// the future.

/// Helper function to swap tokens for exact tokens
pub fn _swap_tokens_for_exact_tokens(
    deps: DepsMut,
    _env: &Env,
    response: Response,
    pairs: Vec<ILbPair>,
    versions: Vec<Version>,
    token_path: Vec<TokenType>,
    _amounts_in: Vec<Uint128>, // only used in V1
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
            let response = response.add_submessage(SubMsg::reply_on_success(
                lb_pair_swap_msg,
                SWAP_FOR_EXACT_REPLY_ID,
            ));

            Ok(response)
        }
    }
}

/// Helper function to return the address of the LBPair.
///
/// Revert if the pair is not created yet.
pub fn _get_lb_pair_information(
    deps: Deps,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
    version: Version,
) -> Result<ContractInfo> {
    // NOTE: We are following the joe-v2 versioning, starting from V2_2.

    if version == Version::V2 {
        unimplemented!()
    } else if version == Version::V2_1 {
        unimplemented!()
    } else {
        let factory = FACTORY.load(deps.storage)?;

        let lb_pair_information =
            factory.get_lb_pair_information(deps.querier, token_x, token_y, bin_step)?;

        Ok(lb_pair_information.lb_pair.contract)
    }

    // TODO: is this possible in our case?
    // if (lbPair == address(0)) {
    //     revert LBRouter__PairNotCreated(address(tokenX), address(tokenY), binStep);
    // }
}

/// Helper function to return the address of the pair (v1 or v2, according to `binStep`).
///
/// Revert if the pair is not created yet.
pub fn _get_pair(
    deps: Deps,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
    version: Version,
) -> Result<ContractInfo> {
    if version == Version::V1 {
        unimplemented!()
    } else {
        _get_lb_pair_information(deps, token_x, token_y, bin_step, version)
    }
}

/// Helper function to return a list of pairs.
pub fn _get_pairs(
    deps: Deps,
    pair_bin_steps: Vec<u16>,
    versions: Vec<Version>,
    token_path: Vec<TokenType>, // contracts that implements the snip20 interface
) -> Result<Vec<ILbPair>> {
    let mut pairs: Vec<ILbPair> = Vec::with_capacity(pair_bin_steps.len());
    let mut token: TokenType;
    let mut token_next = token_path[0].clone();

    for i in 0..pairs.len() {
        token = token_next;
        token_next = token_path[i + 1].clone();

        pairs[i] = ILbPair(_get_pair(
            deps,
            token.clone(),
            token_next.clone(),
            pair_bin_steps[i].clone(),
            versions[i].clone(),
        )?);
    }

    Ok(pairs)
}

// TODO: check out what "safeTransfer" does in ERC20. I think SNIP20 has the same safety already.

/// Helper function to transfer tokens to `to`.
pub fn _safe_transfer(
    token: &TokenType,
    to: &Addr,
    amount: Uint128,
) -> StdResult<Option<CosmosMsg>> {
    if amount == Uint128::zero() {
        return Ok(None);
    }

    // TODO: Handle the TokenType::Native case.

    Ok(Some(snip20::transfer_msg(
        to.to_string(),
        amount,
        None,
        None,
        32,
        token.code_hash().to_string(),
        token.address().to_string(),
    )?))
}

/// Helper function to transfer tokens from `from` to `to`.
pub fn _safe_transfer_from(
    token: &TokenType,
    from: &Addr,
    to: &Addr,
    amount: Uint128,
) -> StdResult<Option<CosmosMsg>> {
    if amount == Uint128::zero() {
        return Ok(None);
    }

    // TODO: Handle the TokenType::Native case.

    Ok(Some(snip20::transfer_from_msg(
        from.to_string(),
        to.to_string(),
        amount,
        None,
        None,
        32,
        token.code_hash().to_string(),
        token.address().to_string(),
    )?))
}

// NOTE: These "native" functions are for working with any token supporting `withdraw` & `redeem`.
// We are highly unlikely to use them on Secret!

/// Helper function to transfer NATIVE to `to`.
pub fn _safe_transfer_native(to: &Addr, amount: Uint128) -> Option<BankMsg> {
    if amount == Uint128::zero() {
        return None;
    }

    // TODO: be able to handle other denoms

    Some(BankMsg::Send {
        to_address: to.to_string(),
        amount: vec![Coin {
            denom: "uscrt".to_string(),
            amount,
        }],
    })
}

#[allow(unused)]

/// Helper function to deposit and transfer WNative to `to`.
pub fn _w_native_deposit_and_transfer(
    to: &Addr,
    amount: Uint128,
) -> StdResult<Option<Vec<CosmosMsg>>> {
    if amount == Uint128::zero() {
        return Ok(None);
    }

    // this information should be stored in the contract, or given as a function argument
    let sscrt: TokenType = todo!();

    let deposit = snip20::deposit_msg(
        amount,
        None,
        32,
        sscrt.code_hash(),
        sscrt.address().to_string(),
    )?;
    let transfer = _safe_transfer(&sscrt, to, amount)?.expect("amount can't be zero");

    Ok(Some(vec![deposit, transfer]))
}

#[allow(unused)]

/// Helper function to withdraw and transfer WNative to `to`.
pub fn _w_native_withdraw_and_transfer(
    to: &Addr,
    amount: Uint128,
) -> StdResult<Option<Vec<CosmosMsg>>> {
    if amount == Uint128::zero() {
        return Ok(None);
    }

    let sscrt: TokenType = todo!();

    let withdraw = snip20::redeem_msg(
        amount,
        Some("uscrt".to_string()),
        None,
        32,
        sscrt.code_hash(),
        sscrt.address().to_string(),
    )?;
    let transfer = _safe_transfer_native(to, amount).expect("amount can't be zero");

    Ok(Some(vec![withdraw, transfer.into()]))
}
