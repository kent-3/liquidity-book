use crate::{contract::FLASH_LOAN_REPLY_ID, helper::*, prelude::*, state::*};
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Binary, ContractInfo, CosmosMsg, Deps, DepsMut, Empty, Env,
    Event, MessageInfo, Response, StdResult, SubMsg, Uint128, Uint256, WasmMsg,
};
use ethnum::U256;
use lb_interfaces::{
    lb_flash_loan_callback,
    lb_pair::{self, *},
    lb_token::{self, LbTokenEventExt},
};
use lb_libraries::{
    constants::PRECISION,
    lb_token::state_structs::{TokenAmount, TokenIdBalance},
    math::{
        tree_math::TREE,
        u24::U24,
        uint256_to_u256::{ConvertU256, ConvertUint256},
    },
    BinHelper, Bytes32, LiquidityConfiguration, OracleMap, PackedUint128Math, PairParameters,
    PriceHelper, U256x256Math,
};
use std::ops::Add;
use std::ops::Div;

static MAX_TOTAL_FEE: u128 = 100_000_000_000_000_000; // 10% of 1e18

#[derive(Clone, Debug)]
pub struct MintArrays {
    pub ids: Vec<u32>,
    pub amounts: Vec<Bytes32>,
    pub liquidity_minted: Vec<U256>,
}

/// Swap tokens iterating over the bins until the entire amount is swapped.
///
/// Token X will be swapped for token Y if `swap_for_y` is true, and token Y for token X if `swap_for_y` is false.
///
/// This function will not transfer the tokens from the caller, it is expected that the tokens have already been
/// transferred to this contract through another contract, most likely the router.
/// That is why this function shouldn't be called directly, but only through one of the swap functions of a router
/// that will also perform safety checks, such as minimum amounts and slippage.
///
/// The variable fee is updated throughout the swap, it increases with the number of bins crossed.
/// The oracle is updated at the end of the swap.
///
/// # Arguments
///
/// * `swap_for_y` - Whether you're swapping token X for token Y (true) or token Y for token X (false)
/// * `to` - The address to send the tokens to
///
/// # Returns
///
/// * `amounts_out` - The encoded amounts of token X and token Y sent to `to`
pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    swap_for_y: bool,
    to: String,
) -> Result<Response> {
    let to = deps.api.addr_validate(&to)?;

    let tree = BIN_TREE.load(deps.storage)?;

    // bytes32 hooksParameters = _hooksParameters;

    let mut reserves = RESERVES.load(deps.storage)?;
    let mut protocol_fees = PROTOCOL_FEES.load(deps.storage)?;

    let mut amounts_out: [u8; 32] = [0; 32];

    let state = STATE.load(deps.storage)?;
    let token_x = TOKEN_X.load(deps.storage)?;
    let token_y = TOKEN_Y.load(deps.storage)?;

    // NOTE: These balance queries should never fail.
    // A few different options here...

    // Most direct way (I kind of prefer, but doesn't work for "native" tokens):
    // let Balance {
    //     amount: token_x_balance,
    // } = deps.querier.query_wasm_smart::<Balance>(
    //     token_x.code_hash(),
    //     token_x.address(),
    //     &snip20::QueryMsg::Balance {
    //         address: env.contract.address.to_string(),
    //         key: state.viewing_key.to_string(),
    //     },
    // )?;
    // let Balance {
    //     amount: token_y_balance,
    // } = deps.querier.query_wasm_smart::<Balance>(
    //     token_y.code_hash(),
    //     token_y.address(),
    //     &snip20::QueryMsg::Balance {
    //         address: env.contract.address.to_string(),
    //         key: state.viewing_key.to_string(),
    //     },
    // )?;

    // Shade way (method on TokenType):
    let token_x_balance = token_x.query_balance(
        deps.as_ref(),
        env.contract.address.to_string(),
        state.viewing_key.to_string(),
    )?;
    let token_y_balance = token_y.query_balance(
        deps.as_ref(),
        env.contract.address.to_string(),
        state.viewing_key.to_string(),
    )?;

    // Internal helper function (not much better because we need so many args):
    // let token_x_balance = _balance_of(
    //     deps.querier,
    //     env,
    //     &token_x.into_contract_info().unwrap(),
    //     state.viewing_key.to_string(),
    // );
    // let token_x_balance = _balance_of(
    //     deps.querier,
    //     env,
    //     &token_x.into_contract_info().unwrap(),
    //     state.viewing_key.to_string(),
    // );

    // secret-toolkit approach (I don't like this as much, but at least it would
    // return an "unaithorized" error instead of a serialization error)
    // there's a typo in secret_toolkit::snip20...
    // let token_x_balance = secret_toolkit::snip20::balance_query(
    //     deps.querier,
    //     env.contract.address.to_string(),
    //     state.viewing_key.to_string(),
    //     1,
    //     token_x.code_hash(),
    //     token_x.address().to_string(),
    // )?;
    // let token_y_balance = secret_toolkit::snip20::balance_query(
    //     deps.querier,
    //     env.contract.address.to_string(),
    //     state.viewing_key.to_string(),
    //     1,
    //     token_y.code_hash(),
    //     token_y.address().to_string(),
    // )?;

    // Yet another approach. Saved just in case.
    // let res = deps
    //     .querier
    //     .query_wasm_smart::<snip20::query::AuthenticatedQueryResponse>(
    //         state.token_x.code_hash(),
    //         state.token_x.address(),
    //         &snip20::QueryMsg::Balance {
    //             address: env.contract.address.to_string(),
    //             key: state.viewing_key.to_string(),
    //         },
    //     )?;
    //
    // let token_x_balance = match res {
    //     snip20::query::AuthenticatedQueryResponse::Balance { amount } => amount,
    //     snip20::query::AuthenticatedQueryResponse::ViewingKeyError { msg } => panic!("{msg}"),
    //     _ => panic!("idk lol"),
    // };
    //
    // let res = deps
    //     .querier
    //     .query_wasm_smart::<snip20::query::AuthenticatedQueryResponse>(
    //         state.token_y.code_hash(),
    //         state.token_y.address(),
    //         &snip20::QueryMsg::Balance {
    //             address: env.contract.address.to_string(),
    //             key: state.viewing_key.to_string(),
    //         },
    //     )?;
    //
    // let token_y_balance = match res {
    //     snip20::query::AuthenticatedQueryResponse::Balance { amount } => amount,
    //     snip20::query::AuthenticatedQueryResponse::ViewingKeyError { msg } => panic!("{msg}"),
    //     _ => panic!("idk lol"),
    // };

    // TODO: write these as methods on Bins, or at least Bytes32
    // example: reserves.received_x(token_x_balance);
    let mut amounts_left = if swap_for_y {
        reserves.received_x(token_x_balance.u128())
    } else {
        reserves.received_y(token_y_balance.u128())
    };
    if amounts_left == [0; 32] {
        return Err(Error::InsufficientAmountIn);
    };

    // Hooks.beforeSwap(hooksParameters, msg.sender, to, swapForY_, amountsLeft);

    reserves = reserves.add(amounts_left);

    let mut parameters = PARAMETERS.load(deps.storage)?;
    let bin_step = BIN_STEP.load(deps.storage)?;

    let mut active_id = parameters.get_active_id();

    parameters.update_references(env.block.time.seconds())?;

    let mut events: Vec<Event> = Vec::new();

    loop {
        let bin_reserves = BIN_MAP
            .load(deps.storage, active_id)
            .map_err(|_| Error::ZeroBinReserve { active_id })?;

        if !bin_reserves.is_empty(!swap_for_y) {
            parameters.update_volatility_accumulator(active_id)?;
            let (mut amounts_in_with_fees, amounts_out_of_bin, total_fees) = bin_reserves
                .get_amounts(parameters, bin_step, swap_for_y, active_id, amounts_left)?;

            if amounts_in_with_fees > [0u8; 32] {
                amounts_left = amounts_left.sub(amounts_in_with_fees);
                amounts_out = amounts_out.add(amounts_out_of_bin);

                let p_fees = total_fees.scalar_mul_div_basis_point_round_down(
                    parameters.get_protocol_share().into(),
                )?;

                if p_fees > [0u8; 32] {
                    protocol_fees = protocol_fees.add(p_fees);
                    amounts_in_with_fees = amounts_in_with_fees.sub(p_fees);
                }

                BIN_MAP.save(
                    deps.storage,
                    active_id,
                    &bin_reserves
                        .add(amounts_in_with_fees)
                        .sub(amounts_out_of_bin),
                )?;

                events.push(Event::swap(
                    info.sender.as_str(),
                    to.as_str(),
                    active_id,
                    amounts_in_with_fees,
                    amounts_out_of_bin,
                    parameters.get_volatility_accumulator(),
                    total_fees,
                    protocol_fees,
                ));
            }
        }

        if amounts_left == [0; 32] {
            break;
        } else {
            let next_id = _get_next_non_empty_bin(deps.storage, swap_for_y, active_id);
            if next_id == 0 || next_id == (U24::MAX) {
                return Err(Error::OutOfLiquidity);
            }
            active_id = next_id;
        }
    }

    if amounts_out == [0; 32] {
        return Err(Error::InsufficientAmountOut);
    }

    RESERVES.save(deps.storage, &reserves.sub(amounts_out))?;
    PROTOCOL_FEES.save(deps.storage, &protocol_fees)?;

    // volume_tracker = volume_tracker.add(amounts_out);

    // TODO: this part is untested
    let mut parameters = ORACLE.update_oracle(
        deps.storage,
        env.block.time.seconds(),
        parameters,
        active_id,
    )?;
    PARAMETERS.save(deps.storage, parameters.set_active_id(active_id)?)?;

    // SAFETY: We checked earlier that amounts_out > 0, so
    // these bin transfer functions will always return Some.
    let msg = if swap_for_y {
        bin_transfer_y(amounts_out, token_y.clone(), to)
    } else {
        bin_transfer_x(amounts_out, token_x.clone(), to)
    }
    .expect("there must be a transfer message");

    Ok(Response::new().add_message(msg).add_events(events))
}

/// Flash loan tokens from the pool to a receiver contract and execute a callback function.
/// The receiver contract is expected to return the tokens plus a fee to this contract.
/// The fee is calculated as a percentage of the amount borrowed, and is the same for both tokens.
///
/// # Arguments
///
/// * `receiver` - The contract that will receive the tokens and execute the callback function
/// * `amounts` - The encoded amounts of token X and token Y to flash loan
/// * `data` - Any data that will be passed to the callback function
pub fn flash_loan(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    receiver: ContractInfo,
    amounts: Bytes32,
    data: Option<Binary>,
) -> Result<Response> {
    if amounts == [0u8; 32] {
        return Err(Error::ZeroBorrowAmount);
    }

    let hooks_parameters = HOOKS_PARAMETERS.load(deps.storage)?;

    let reserves_before = RESERVES.load(deps.storage)?;
    let total_fees = _get_flash_loan_fees(deps.as_ref(), amounts)?;

    // TODO: Hooks
    //     Hooks.beforeFlashLoan(hooksParameters, msg.sender, address(receiver), amounts);

    // TODO: transfer the requested token amounts to the receiver
    //     amounts.transfer(_tokenX(), _tokenY(), address(receiver));

    // TODO: Create a SubMsg to execute the LBFlashLoanCallback message on the receiver contract.
    //     (bool success, bytes memory rData) = address(receiver).call(
    //         abi.encodeWithSelector(
    //             ILBFlashLoanCallback.LBFlashLoanCallback.selector,
    //             msg.sender,
    //             _tokenX(),
    //             _tokenY(),
    //             amounts,
    //             totalFees,
    //             data
    //         )
    //     );
    //

    // TODO: how to handle the native token case?
    let token_x = TOKEN_X.load(deps.storage)?.into_contract_info().unwrap();
    let token_y = TOKEN_Y.load(deps.storage)?.into_contract_info().unwrap();

    let msg = lb_flash_loan_callback::ExecuteMsg::LbFlashLoanCallback {
        address: info.sender.to_string(),
        token_x,
        token_y,
        amounts,
        total_fees,
        data,
    };

    // TODO: what does setting Empty do exactly?
    let msg = SubMsg::<Empty>::reply_on_success(
        WasmMsg::Execute {
            contract_addr: receiver.address.to_string(),
            code_hash: receiver.code_hash,
            msg: to_binary(&msg)?,
            funds: vec![],
        },
        FLASH_LOAN_REPLY_ID,
    );

    EPHEMERAL_FLASH_LOAN.save(
        deps.storage,
        &EphemeralFlashLoan {
            reserves_before,
            total_fees,
            sender: info.sender,
            receiver: receiver.address,
            amounts,
        },
    )?;

    let response = Response::new().add_submessage(msg);

    Ok(response)

    // TODO: Handle the rest of this function in the Reply.
}

/// Returns the encoded fees amounts for a flash loan
///
/// # Arguments
///
/// * `amounts` - The amounts of the flash loan
pub fn _get_flash_loan_fees(deps: Deps, amounts: Bytes32) -> Result<Bytes32> {
    let fee = FACTORY
        .load(deps.storage)?
        .get_flash_loan_fee(deps.querier)?;
    let (x, y) = amounts.decode();

    // TODO: Double check this math.
    // I think we can avoid some of the checks because PRECISION is constant.
    let precision_sub_one = Uint256::from(PRECISION - 1);
    let x: Uint128 = Uint128::new(x)
        .full_mul(fee)
        .add(precision_sub_one)
        .div(Uint256::from(PRECISION))
        .try_into()?;
    let y: Uint128 = Uint128::new(y)
        .full_mul(fee)
        .add(precision_sub_one)
        .div(Uint256::from(PRECISION))
        .try_into()?;

    //     unchecked {
    //         uint256 precisionSubOne = Constants.PRECISION - 1;
    //         x = ((uint256(x) * fee + precisionSubOne) / Constants.PRECISION).safe128();
    //         y = ((uint256(y) * fee + precisionSubOne) / Constants.PRECISION).safe128();
    //     }

    Ok(PackedUint128Math::encode(x.u128(), y.u128()))
}

/// Mint liquidity tokens by depositing tokens into the pool.
///
/// It will mint Liquidity Book (LB) tokens for each bin where the user adds liquidity.
/// This function will not transfer the tokens from the caller, it is expected that the tokens have already been
/// transferred to this contract through another contract, most likely the router.
/// That is why this function shouldn't be called directly, but through one of the add liquidity functions of a
/// router that will also perform safety checks.
///
/// Any excess amount of token will be sent to the `refund_to` address.
///
/// # Arguments
///
/// * `to` - The address that will receive the LB tokens
/// * `liquidity_configs` - The encoded liquidity configurations, each one containing the id of the bin and the
///   percentage of token X and token Y to add to the bin.
/// * `refund_to` - The address that will receive the excess amount of tokens
///
/// # Returns
///
/// * `amounts_received` - The amounts of token X and token Y received by the pool
/// * `amounts_left` - The amounts of token X and token Y that were not added to the pool and were sent to to
/// * `liquidity_minted` - The amounts of LB tokens minted for each bin
pub fn mint(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to: String,
    liquidity_configs: Vec<LiquidityConfiguration>,
    refund_to: String,
) -> Result<Response> {
    if liquidity_configs.is_empty() {
        return Err(Error::EmptyMarketConfigs);
    }

    let to = deps.api.addr_validate(&to)?;
    let refund_to = deps.api.addr_validate(&refund_to)?;

    // TODO: these don't need to be U256 types. I think they just do that in EVM land.
    // ids should be u24/u32
    // liquidity_minted might still be U256 or Uint256
    let mut arrays = MintArrays {
        ids: vec![0u32; liquidity_configs.len()],
        amounts: vec![[0u8; 32]; liquidity_configs.len()],
        liquidity_minted: vec![U256::ZERO; liquidity_configs.len()],
    };

    let reserves = RESERVES.load(deps.storage)?;

    let state = STATE.load(deps.storage)?;
    let token_x = TOKEN_X.load(deps.storage)?;
    let token_y = TOKEN_Y.load(deps.storage)?;

    let token_x_balance = token_x.query_balance(
        deps.as_ref(),
        env.contract.address.to_string(),
        state.viewing_key.to_string(),
    )?;
    let token_y_balance = token_y.query_balance(
        deps.as_ref(),
        env.contract.address.to_string(),
        state.viewing_key.to_string(),
    )?;

    let amounts_received = reserves.received(token_x_balance.u128(), token_y_balance.u128());

    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut events: Vec<Event> = Vec::new();

    let amounts_left = mint_bins(
        &mut deps,
        &env,
        &info,
        &mut messages,
        &mut events,
        liquidity_configs,
        amounts_received,
        to.clone(),
        &mut arrays,
    )?;

    RESERVES.save(
        deps.storage,
        &reserves.add(amounts_received.sub(amounts_left)),
    )?;

    let liquidity_minted = arrays
        .liquidity_minted
        .into_iter()
        .map(|el| el.u256_to_uint256())
        .collect();

    events.extend(vec![
        // TODO: Decide what to use for the 'from' address.
        // NOTE: 2nd argument is the 'from' address. In EVM they use address(0).
        // Since tokens are being minted, they aren't really coming 'from' anywhere.
        Event::transfer_batch(
            &info.sender,
            &Addr::unchecked("0"),
            &to,
            &arrays.ids,
            &liquidity_minted,
        ),
        Event::deposited_to_bins(&info.sender, &to, &arrays.ids, &arrays.amounts),
    ]);

    // TODO: the check for amounts > 0 happens internally as well, which is annoying
    let refund_messages = if amounts_left > [0u8; 32] {
        bin_transfer(amounts_left, token_x, token_y, refund_to)
    } else {
        vec![]
    };

    let data = lb_pair::MintResponse {
        amounts_received,
        amounts_left,
        liquidity_minted,
    };

    let response = Response::new()
        .set_data(to_binary(&data)?)
        .add_events(events)
        .add_messages(messages)
        .add_messages(refund_messages);

    Ok(response)
}

/// Helper function to mint liquidity in each bin in the liquidity configurations.
///
/// # Arguments
///
/// * `liquidity_configs` - The liquidity configurations.
/// * `amounts_received` - The amounts received.
/// * `to` - The address to mint the liquidity to.
/// * `arrays` - The arrays to store the results.
///
/// # Returns
///
/// * `amounts_left` - The amounts left.
fn mint_bins(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    messages: &mut Vec<CosmosMsg>,
    events: &mut Vec<Event>,
    liquidity_configs: Vec<LiquidityConfiguration>,
    amounts_received: Bytes32,
    to: Addr,
    arrays: &mut MintArrays,
) -> Result<Bytes32> {
    let bin_step = BIN_STEP.load(deps.storage)?;

    let parameters = PARAMETERS.load(deps.storage)?;
    let active_id = parameters.get_active_id();

    let mut amounts_left = amounts_received;

    let mut mint_tokens: Vec<TokenAmount> = Vec::new();

    for (i, liq_conf) in liquidity_configs.iter().enumerate() {
        let (max_amounts_in_to_bin, id) = liq_conf.get_amounts_and_id(amounts_received)?;
        let (shares, amounts_in, amounts_in_to_bin) = update_bin(
            deps,
            env,
            info,
            events,
            bin_step,
            active_id,
            id,
            max_amounts_in_to_bin,
            parameters,
        )?;

        amounts_left = amounts_left.sub(amounts_in);

        arrays.ids[i] = id;
        arrays.amounts[i] = amounts_in_to_bin;
        arrays.liquidity_minted[i] = shares;

        mint_tokens.push(TokenAmount {
            token_id: id.to_string(),
            balances: vec![TokenIdBalance {
                address: to.clone(),
                amount: shares.u256_to_uint256(),
            }],
        });
    }

    let lb_token = LB_TOKEN.load(deps.storage)?;

    let mint_tokens_msg = wasm_execute(
        lb_token.address,
        lb_token.code_hash,
        &lb_token::ExecuteMsg::MintTokens {
            mint_tokens,
            memo: None,
            padding: None,
        },
        vec![],
    )?;

    messages.push(mint_tokens_msg.into());

    Ok(amounts_left)
}

/// Helper function to update a bin during minting.
///
/// # Arguments
///
/// * `bin_step` - The bin step of the pair
/// * `active_id` - The id of the active bin
/// * `id` - The id of the bin
/// * `max_amounts_in_to_bin` - The maximum amounts in to the bin
/// * `parameters` - The parameters of the pair
///
/// # Returns
///
/// * `shares` - The amount of shares minted
/// * `amounts_in` - The amounts in
/// * `amounts_in_to_bin` - The amounts in to the bin
fn update_bin(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    events: &mut Vec<Event>,
    bin_step: u16,
    active_id: u32,
    id: u32,
    amounts_in: Bytes32,
    mut parameters: PairParameters,
) -> Result<(U256, Bytes32, Bytes32)> {
    let bin_reserves = BIN_MAP.load(deps.storage, id).unwrap_or_default();

    let price = PriceHelper::get_price_from_id(id, bin_step)?;
    let supply = _query_total_supply(deps.as_ref(), id)?;

    let (mut shares, amounts_in) =
        bin_reserves.get_shares_and_effective_amounts_in(amounts_in, price, supply)?;
    let amounts_in_to_bin = amounts_in;

    if id == active_id {
        parameters.update_volatility_parameters(id, env.block.time.seconds())?;

        // Helps calculate fee if there's an implict swap.
        let fees =
            bin_reserves.get_composition_fees(parameters, bin_step, amounts_in, supply, shares)?;

        if fees != [0u8; 32] {
            let user_liquidity = amounts_in.sub(fees).get_liquidity(price)?;
            let protocol_c_fees =
                fees.scalar_mul_div_basis_point_round_down(parameters.get_protocol_share().into())?;

            if protocol_c_fees != [0u8; 32] {
                let _amounts_in_to_bin = amounts_in_to_bin.sub(protocol_c_fees);
                PROTOCOL_FEES.update(deps.storage, |mut p_fees| -> StdResult<_> {
                    p_fees = p_fees.add(protocol_c_fees);
                    Ok(p_fees)
                })?;
            }

            let bin_liquidity = bin_reserves
                .add(fees.sub(protocol_c_fees))
                .get_liquidity(price)?;
            shares = user_liquidity.mul_div_round_down(supply, bin_liquidity)?;
            // shares = U256x256Math::mul_div_round_down(user_liquidity, supply, bin_liquidity)?;

            parameters =
                ORACLE.update_oracle(deps.storage, env.block.time.seconds(), parameters, id)?;
            PARAMETERS.save(deps.storage, &parameters)?;

            events.push(Event::composition_fees(
                &info.sender,
                id,
                &fees,
                &protocol_c_fees,
            ));
        }
    } else {
        amounts_in.verify_amounts(active_id, id)?;
    }

    if shares == 0 || amounts_in_to_bin == [0u8; 32] {
        return Err(Error::ZeroAmount { id });
    }

    if supply == 0 {
        TREE.add(deps.storage, id);
        // BIN_TREE.update(deps.storage, |mut tree| -> StdResult<_> {
        //     tree.add(id);
        //     Ok(tree)
        // })?;
    }

    BIN_MAP.save(deps.storage, id, &bin_reserves.add(amounts_in_to_bin))?;

    Ok((shares, amounts_in, amounts_in_to_bin))
}

// TODO: review this function!

/// Burn Liquidity Book (LB) tokens and withdraw tokens from the pool.
///
/// This function will burn the tokens directly from the caller.
///
/// # Arguments
///
/// * `from` - The address that will burn the LB tokens
/// * `to` - The address that will receive the tokens
/// * `ids` - The ids of the bins from which to withdraw
/// * `amounts_to_burn` - The amounts of LB tokens to burn for each bin
///
/// # Returns
///
/// * `amounts` - The amounts of token X and token Y received by the user
pub fn burn(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: String,
    to: String,
    ids: Vec<u32>,
    amounts_to_burn: Vec<Uint256>,
) -> Result<Response> {
    if ids.is_empty() || ids.len() != amounts_to_burn.len() {
        return Err(Error::InvalidInput);
    }

    // bytes32 hooksParameters = _hooksParameters;
    //
    // Hooks.beforeBurn(hooksParameters, msg.sender, from, to, ids, amountsToBurn);

    let from = deps.api.addr_validate(&from)?;
    let to = deps.api.addr_validate(&to)?;

    let token_x = TOKEN_X.load(deps.storage)?;
    let token_y = TOKEN_Y.load(deps.storage)?;

    let mut burn_tokens: Vec<TokenAmount> = Vec::new();

    let mut amounts = vec![[0u8; 32]; ids.len()];

    let mut amounts_out = [0u8; 32];

    for i in 0..ids.len() {
        let id = ids[i];
        let amount_to_burn = amounts_to_burn[i];

        if amount_to_burn.is_zero() {
            return Err(Error::ZeroShares { id });
        }

        // TODO: this error doesn't seem right. maybe a let-else would make more sense?
        let bin_reserves = BIN_MAP
            .load(deps.storage, id)
            .map_err(|_| Error::ZeroBinReserve {
                active_id: i as u32,
            })?;

        // TODO: starting to wonder if using this U256 type everywhere is the right approach...
        let supply = _query_total_supply(deps.as_ref(), id)?;

        burn_tokens.push(TokenAmount {
            token_id: id.to_string(),
            balances: vec![TokenIdBalance {
                address: from.clone(),
                amount: amount_to_burn,
            }],
        });

        let amounts_out_from_bin =
            bin_reserves.get_amount_out_of_bin(amount_to_burn.uint256_to_u256(), supply)?;

        if amounts_out_from_bin == [0u8; 32] {
            return Err(Error::ZeroAmountsOut {
                id,
                amount_to_burn,
                total_supply: supply.u256_to_uint256(),
            });
        }

        let bin_reserves = bin_reserves.sub(amounts_out_from_bin);

        if supply == amount_to_burn.uint256_to_u256() {
            TREE.remove(deps.storage, id);
            // BIN_TREE.update(deps.storage, |mut tree| -> StdResult<_> {
            //     tree.remove(id);
            //     Ok(tree)
            // })?;
        }

        BIN_MAP.save(deps.storage, id, &bin_reserves)?;
        amounts[i] = amounts_out_from_bin;
        amounts_out = amounts_out.add(amounts_out_from_bin);
    }

    let lb_token = LB_TOKEN.load(deps.storage)?;

    let burn_tokens_msg = wasm_execute(
        lb_token.address,
        lb_token.code_hash,
        &lb_token::ExecuteMsg::BurnTokens {
            burn_tokens,
            memo: None,
            padding: None,
        },
        vec![],
    )?;

    RESERVES.update(deps.storage, |reserves| -> StdResult<Bytes32> {
        Ok(reserves.sub(amounts_out))
    })?;

    // TODO: events
    // emit TransferBatch(msg.sender, from_, address(0), ids, amountsToBurn);
    // emit WithdrawnFromBins(msg.sender, to, ids, amounts);

    let transfer_messages = bin_transfer(amounts_out, token_x, token_y, to);

    let response_data = to_binary(&BurnResponse { amounts })?;

    Ok(Response::default()
        .set_data(response_data)
        .add_message(burn_tokens_msg)
        .add_messages(transfer_messages))

    // the burn and transfer messages are "fire and forget"
    // the amounts burned go back to the router contract that called this execute message
}

// Administrative functions

/// Collect the protocol fees from the pool.
pub fn collect_protocol_fees(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response> {
    let recipient = FACTORY
        .load(deps.storage)?
        .get_fee_recipient(deps.querier)?;

    if info.sender != recipient {
        return Err(Error::OnlyProtocolFeeRecipient);
    }

    let reserves = RESERVES.load(deps.storage)?;
    let protocol_fees = PROTOCOL_FEES.load(deps.storage)?;

    let token_x = TOKEN_X.load(deps.storage)?;
    let token_y = TOKEN_Y.load(deps.storage)?;

    let (x, y) = protocol_fees.decode();
    let ones = Bytes32::encode(if x > 0 { 1 } else { 0 }, if y > 0 { 1 } else { 0 });

    //The purpose of subtracting ones from the protocolFees is to leave a small amount (1 unit of each token) in the protocol fees.
    //This is done to avoid completely draining the fees and possibly causing any issues with calculations that depend on non-zero values
    let collected_protocol_fees = protocol_fees.sub(ones);

    if collected_protocol_fees != [0u8; 32] {
        // This is setting the protocol fees to the smallest possible values
        PROTOCOL_FEES.save(deps.storage, &ones)?;
        RESERVES.save(deps.storage, &reserves.sub(collected_protocol_fees))?;

        let event = Event::collected_protocol_fees(&info.sender, &collected_protocol_fees);

        let transfer_messages = bin_transfer(
            collected_protocol_fees,
            token_x.clone(),
            token_y.clone(),
            info.sender.clone(),
        );

        Ok(Response::new()
            .set_data(collected_protocol_fees)
            .add_event(event)
            .add_messages(transfer_messages))
    } else {
        Ok(Response::new().set_data(collected_protocol_fees)) // returning [0u8;32]
    }
}

/// Increase the length of the oracle used by the pool.
///
/// # Arguments
///
/// * `new_length` - The new length of the oracle
pub fn increase_oracle_length(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_length: u16,
) -> Result<Response> {
    let mut parameters = PARAMETERS.load(deps.storage)?;

    let mut oracle_id = parameters.get_oracle_id();

    // activate the oracle if it is not active yet
    if oracle_id == 0 {
        oracle_id = 1;
        PARAMETERS.save(deps.storage, parameters.set_oracle_id(oracle_id))?;
    }

    ORACLE.increase_length(deps.storage, oracle_id, new_length)?;

    let event = Event::oracle_length_increased(&info.sender, new_length);

    Ok(Response::new().add_event(event))
}

/// Sets the static fee parameters of the pool.
///
/// Can only be called by the factory.
///
/// # Arguments
///
/// * `base_factor` - The base factor of the static fee
/// * `filter_period` - The filter period of the static fee
/// * `decay_period` - The decay period of the static fee
/// * `reduction_factor` - The reduction factor of the static fee
/// * `variable_fee_control` - The variable fee control of the static fee
/// * `protocol_share` - The protocol share of the static fee
/// * `max_volatility_accumulator` - The max volatility accumulator of the static fee
pub fn set_static_fee_parameters(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
) -> Result<Response> {
    let factory = FACTORY.load(deps.storage)?;
    only_factory(&info.sender, &factory.address)?;

    if base_factor == 0
        && filter_period == 0
        && decay_period == 0
        && reduction_factor == 0
        && variable_fee_control == 0
        && protocol_share == 0
        && max_volatility_accumulator == 0
    {
        return Err(Error::InvalidStaticFeeParameters);
    }

    let mut parameters = PARAMETERS.load(deps.storage)?;

    parameters.set_static_fee_parameters(
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    )?;

    {
        let mut parameters = parameters.clone();
        let bin_step = BIN_STEP.load(deps.storage)?;
        let max_parameters = parameters.set_volatility_accumulator(max_volatility_accumulator)?;
        let total_fee =
            max_parameters.get_base_fee(bin_step) + max_parameters.get_variable_fee(bin_step);
        if total_fee > MAX_TOTAL_FEE {
            return Err(Error::MaxTotalFeeExceeded {});
        }
    }

    PARAMETERS.save(deps.storage, &parameters)?;

    let event = Event::static_fee_parameters_set(
        &info.sender,
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    );

    Ok(Response::new().add_event(event))
}

/// Forces the decay of the volatility reference variables.
///
/// Can only be called by the factory.
pub fn force_decay(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response> {
    only_factory(&info.sender, &FACTORY.load(deps.storage)?.address)?;

    let mut paramaters = PARAMETERS.load(deps.storage)?;

    PARAMETERS.save(
        deps.storage,
        paramaters
            .update_id_reference()
            .update_volatility_reference()?,
    )?;

    let event = Event::forced_decay(
        &info.sender,
        paramaters.get_id_reference(),
        paramaters.get_volatility_reference(),
    );

    Ok(Response::new().add_event(event))
}

// TODO:
/**
 * @notice Sets the hooks parameter of the pool
 * @dev Can only be called by the factory
 * @param hooksParameters The hooks parameter
 * @param onHooksSetData The data to be passed to the onHooksSet function of the hooks contract
 */
#[allow(unused)]
pub fn set_hooks_parameters(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    hooks_parameters: Bytes32,
    on_hooks_set_data: Binary,
) -> Result<Response> {
    let hooks_parameters = HOOKS_PARAMETERS.load(deps.storage)?;

    todo!();

    //     ILBHooks hooks = ILBHooks(Hooks.getHooks(hooksParameters));

    let event = Event::hooks_parameters_set(&info.sender, &hooks_parameters);

    //     if (address(hooks) != address(0) && hooks.getLBPair() != this) revert LBPair__InvalidHooks();
    //
    //     Hooks.onHooksSet(hooksParameters, onHooksSetData);

    Ok(Response::new().add_event(event))
}

// TODO:
/**
 * @notice Overrides the batch transfer function to call the hooks before and after the transfer
 * @param from The address to transfer from
 * @param to The address to transfer to
 * @param ids The ids of the tokens to transfer
 * @param amounts The amounts of the tokens to transfer
 */
#[allow(unused)]
pub fn batch_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: String,
    to: String,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> Result<Response> {
    let hooks_parameters = HOOKS_PARAMETERS.load(deps.storage)?;

    //     Hooks.beforeBatchTransferFrom(hooksParameters, msg.sender, from, to, ids, amounts);

    let lb_token = LB_TOKEN.load(deps.storage)?;
    // TODO: will this allow a "BatchTransferFrom" type of message?
    let msg = lb_token::ExecuteMsg::BatchTransfer {
        actions: todo!(),
        padding: None,
    }
    .to_cosmos_msg(lb_token.code_hash, lb_token.address.to_string(), None)?;

    //     LBToken.batchTransferFrom(from, to, ids, amounts);
    //
    //     Hooks.afterBatchTransferFrom(hooksParameters, msg.sender, from, to, ids, amounts);

    Ok(Response::new().add_message(msg))
}

pub fn approx_div(a: Uint256, b: Uint256) -> Uint256 {
    if b == Uint256::zero() {
        panic!("Division by zero");
    }
    let div = a / b;
    let rem = a % b;
    if rem >= b / Uint256::from(2u128) {
        // If so, we add one to the division result
        div + Uint256::one()
    } else {
        // If not, we return the division result as it is
        div
    }
}
