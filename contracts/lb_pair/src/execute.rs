use crate::{helper::*, prelude::*, state::*};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128,
    Uint256,
};
use ethnum::U256;
use lb_interfaces::{
    lb_pair::{self, *},
    lb_token,
};
use lb_libraries::{
    bin_helper::BinHelper,
    constants::MAX_FEE,
    lb_token::state_structs::{TokenAmount, TokenIdBalance},
    math::{
        liquidity_configurations::LiquidityConfiguration,
        packed_u128_math::PackedUint128Math,
        u24::U24,
        u256x256_math::U256x256Math,
        uint256_to_u256::{ConvertU256, ConvertUint256},
    },
    oracle_helper::OracleMap,
    pair_parameter_helper::PairParameters,
    price_helper::PriceHelper,
    types::Bytes32,
};
use secret_toolkit::snip20::{self, query::Balance};
use shade_protocol::swap::core::TokenType;

#[derive(Clone, Debug)]
pub struct MintArrays {
    pub ids: Vec<U256>,
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
    _info: MessageInfo,
    swap_for_y: bool,
    to: String,
    // amounts_received: Uint128, //Will get this parameter from router contract
) -> Result<Response> {
    let to = deps.api.addr_validate(&to)?;

    let state = STATE.load(deps.storage)?;
    let tree = BIN_TREE.load(deps.storage)?;
    let token_x = &state.token_x;
    let token_y = &state.token_y;
    let reserves = state.reserves;
    let mut protocol_fees = state.protocol_fees;

    let mut ids = Vec::new();

    // Logging the swap activity
    let mut total_fees: [u8; 32] = [0; 32];
    let mut lp_fees: [u8; 32] = [0; 32];
    let mut shade_dao_fees: [u8; 32] = [0; 32];

    let mut amounts_out: [u8; 32] = [0; 32];

    // TODO: SOMEDAY we could support NativeToken types by querying the denom from bank module.

    // TODO: This could be written much better. Have the BinHelper::received* functions take in
    // the ContractInfo and viewing_key, and check the balances there. However, that means the
    // library now has to know about cosmwasm...

    // NOTE: These should never fail because the pair contract has it's viewing_key saved.

    let Balance {
        amount: token_x_balance,
    } = deps.querier.query_wasm_smart::<Balance>(
        state.token_x.code_hash(),
        state.token_x.address(),
        &snip20::QueryMsg::Balance {
            address: env.contract.address.to_string(),
            key: state.viewing_key.to_string(),
        },
    )?;

    let Balance {
        amount: token_y_balance,
    } = deps.querier.query_wasm_smart::<Balance>(
        state.token_y.code_hash(),
        state.token_y.address(),
        &snip20::QueryMsg::Balance {
            address: env.contract.address.to_string(),
            key: state.viewing_key.to_string(),
        },
    )?;

    let mut amounts_left: [u8; 32] = if swap_for_y {
        BinHelper::received_x(reserves, token_x_balance.u128())
    } else {
        BinHelper::received_y(reserves, token_y_balance.u128())
    };
    if amounts_left == [0; 32] {
        return Err(Error::InsufficientAmountIn);
    };

    let mut volume_tracker = amounts_left;
    let mut reserves = reserves.add(amounts_left);
    let mut params = state.pair_parameters;
    let bin_step = state.bin_step;
    let mut active_id = params.get_active_id();

    // updating the volatility
    params.update_references(env.block.time.seconds())?;

    loop {
        let bin_reserves = BIN_MAP
            .load(deps.storage, active_id)
            .map_err(|_| Error::ZeroBinReserve { active_id })?;

        if !BinHelper::is_empty(bin_reserves, !swap_for_y) {
            let price = PriceHelper::get_price_from_id(active_id, bin_step)?;
            params.update_volatility_accumulator(active_id)?;
            let (mut amounts_in_with_fees, amounts_out_of_bin, fees) = BinHelper::get_amounts(
                bin_reserves,
                params,
                bin_step,
                swap_for_y,
                amounts_left,
                price,
            )?;

            if U256::from_le_bytes(amounts_in_with_fees) > U256::ZERO {
                amounts_left = amounts_left.sub(amounts_in_with_fees);
                amounts_out = amounts_out.add(amounts_out_of_bin);

                let p_fees =
                    fees.scalar_mul_div_basis_point_round_down(params.get_protocol_share().into())?;
                total_fees = total_fees.add(fees);
                lp_fees = lp_fees.add(fees.sub(p_fees));
                shade_dao_fees = shade_dao_fees.add(p_fees);

                if U256::from_le_bytes(p_fees) > U256::ZERO {
                    protocol_fees = protocol_fees.add(p_fees);
                    amounts_in_with_fees = amounts_in_with_fees.sub(p_fees);
                }

                BIN_MAP.save(
                    deps.storage,
                    active_id,
                    &bin_reserves
                        .add(amounts_in_with_fees) // actually amount in wihtout fees
                        .sub(amounts_out_of_bin),
                )?;
                ids.push(active_id);
            }
        }

        if amounts_left == [0; 32] {
            break;
        } else {
            let next_id = _get_next_non_empty_bin(&tree, swap_for_y, active_id);
            if next_id == 0 || next_id == (U24::MAX) {
                return Err(Error::OutOfLiquidity);
            }
            active_id = next_id;
        }
    }

    if amounts_out == [0; 32] {
        return Err(Error::InsufficientAmountOut);
    }

    reserves = reserves.sub(amounts_out);
    volume_tracker = volume_tracker.add(amounts_out);

    // updating_oracles_for_vol_analysis(
    //     deps.storage,
    //     &env,
    //     &mut params,
    //     active_id,
    //     volume_tracker,
    //     total_fees,
    // )?;

    STATE.update(deps.storage, |mut state| {
        state.last_swap_timestamp = env.block.time;
        state.protocol_fees = protocol_fees;
        params
            .set_active_id(active_id)
            .map_err(|err| StdError::generic_err(err.to_string()))?;
        state.pair_parameters = params;
        state.reserves = reserves;
        Ok::<State, StdError>(state)
    })?;

    let mut messages: Vec<CosmosMsg> = Vec::new();
    // Determine the output amount and the corresponding transfer message based on swap_for_y
    let amount_out = if swap_for_y {
        amounts_out.decode_y()
    } else {
        amounts_out.decode_x()
    };
    let msg = if swap_for_y {
        bin_transfer_y(amounts_out, token_y.clone(), to)
    } else {
        bin_transfer_x(amounts_out, token_x.clone(), to)
    };
    // Add the message to messages if it exists
    if let Some(message) = msg {
        messages.push(message);
    }

    Ok(Response::new().add_messages(messages))
    // TODO: rethink attributes and data
    // .add_attributes(vec![
    //     Attribute::new("amount_in", amounts_received),
    //     Attribute::new("amount_out", amount_out.to_string()),
    //     Attribute::new("lp_fee_amount", lp_fees.decode_alt(swap_for_y).to_string()),
    //     Attribute::new(
    //         "total_fee_amount",
    //         total_fees.decode_alt(swap_for_y).to_string(),
    //     ),
    //     Attribute::new(
    //         "shade_dao_fee_amount",
    //         shade_dao_fees.decode_alt(swap_for_y).to_string(),
    //     ),
    //     Attribute::new("token_in_key", token_x.unique_key()),
    //     Attribute::new("token_out_key", token_y.unique_key()),
    // ])
    // .set_data(to_binary(&ExecuteMsgResponse::SwapResult {
    //     amount_out: Uint128::from(amount_out),
    // })?))
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

    let state = STATE.load(deps.storage)?;

    let res = deps
        .querier
        .query_wasm_smart::<snip20::query::AuthenticatedQueryResponse>(
            state.token_x.code_hash(),
            state.token_x.address(),
            &snip20::QueryMsg::Balance {
                address: env.contract.address.to_string(),
                key: state.viewing_key.to_string(),
            },
        )?;

    let token_x_balance = match res {
        snip20::query::AuthenticatedQueryResponse::Balance { amount } => amount,
        snip20::query::AuthenticatedQueryResponse::ViewingKeyError { msg } => panic!("{msg}"),
        _ => panic!("idk lol"),
    };

    let res = deps
        .querier
        .query_wasm_smart::<snip20::query::AuthenticatedQueryResponse>(
            state.token_y.code_hash(),
            state.token_y.address(),
            &snip20::QueryMsg::Balance {
                address: env.contract.address.to_string(),
                key: state.viewing_key.to_string(),
            },
        )?;

    let token_y_balance = match res {
        snip20::query::AuthenticatedQueryResponse::Balance { amount } => amount,
        snip20::query::AuthenticatedQueryResponse::ViewingKeyError { msg } => panic!("{msg}"),
        _ => panic!("idk lol"),
    };

    // let Balance {
    //     amount: token_x_balance,
    // } = deps.querier.query_wasm_smart::<Balance>(
    //     state.token_x.code_hash(),
    //     state.token_x.address(),
    //     &snip20::QueryMsg::Balance {
    //         address: env.contract.address.to_string(),
    //         key: state.viewing_key.to_string(),
    //     },
    // )?;

    // let Balance {
    //     amount: token_y_balance,
    // } = deps.querier.query_wasm_smart::<Balance>(
    //     state.token_y.code_hash(),
    //     state.token_y.address(),
    //     &snip20::QueryMsg::Balance {
    //         address: env.contract.address.to_string(),
    //         key: state.viewing_key.to_string(),
    //     },
    // )?;

    let mut mint_arrays = MintArrays {
        ids: (vec![U256::ZERO; liquidity_configs.len()]),
        amounts: (vec![[0u8; 32]; liquidity_configs.len()]),
        liquidity_minted: (vec![U256::ZERO; liquidity_configs.len()]),
    };

    let amounts_received = BinHelper::received(
        state.reserves,
        token_x_balance.u128(),
        token_y_balance.u128(),
    );

    let mut messages: Vec<CosmosMsg> = Vec::new();

    let amounts_left = mint_bins(
        &mut deps,
        env.block.time.seconds(),
        state.bin_step,
        state.pair_parameters,
        liquidity_configs,
        amounts_received,
        to,
        &mut mint_arrays,
        &mut messages,
    )?;

    STATE.update(deps.storage, |mut state| -> StdResult<_> {
        state.reserves = state.reserves.add(amounts_received.sub(amounts_left)); //Total liquidity of pool
        Ok(state)
    })?;

    let (amount_left_x, amount_left_y) = amounts_left.decode();

    // TODO: is this for the refund? rewrite this to not be an iterator...

    let mut transfer_messages = Vec::new();
    // 2- tokens checking and transfer
    for (token, amount) in [
        (
            state.token_x.clone(),
            Uint128::from(amounts_received.decode_x() - amount_left_x),
        ),
        (
            state.token_y.clone(),
            Uint128::from(amounts_received.decode_y() - amount_left_y),
        ),
    ]
    .iter()
    {
        match token {
            TokenType::CustomToken {
                contract_addr: _,
                token_code_hash: _,
            } => {
                let msg =
                    token.transfer_from(*amount, info.sender.clone(), env.contract.address.clone());

                if let Some(m) = msg {
                    transfer_messages.push(m);
                }
            }
            TokenType::NativeToken { .. } => {
                token.assert_sent_native_token_balance(&info, *amount)?;
            }
        }
    }

    let liquidity_minted = mint_arrays
        .liquidity_minted
        .into_iter()
        .map(|el| el.u256_to_uint256())
        .collect();

    let data = lb_pair::MintResponse {
        amounts_received,
        amounts_left,
        liquidity_minted,
    };

    let response = Response::new()
        .set_data(to_binary(&data)?)
        .add_messages(messages)
        .add_messages(transfer_messages);

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
    time: u64,
    bin_step: u16,
    pair_parameters: PairParameters,
    liquidity_configs: Vec<LiquidityConfiguration>,
    amounts_received: Bytes32,
    to: Addr,
    mint_arrays: &mut MintArrays,
    messages: &mut Vec<CosmosMsg>,
) -> Result<Bytes32> {
    let config = STATE.load(deps.storage)?;
    let active_id = pair_parameters.get_active_id();

    let mut amounts_left = amounts_received;

    //Minting tokens

    let mut mint_tokens: Vec<TokenAmount> = Vec::new();

    for (index, liq_conf) in liquidity_configs.iter().enumerate() {
        let (max_amounts_in_to_bin, id) = liq_conf.get_amounts_and_id(amounts_received)?;

        let (shares, amounts_in, amounts_in_to_bin) = update_bin(
            deps,
            time,
            bin_step,
            active_id,
            id,
            max_amounts_in_to_bin,
            pair_parameters,
        )?;

        amounts_left = amounts_left.sub(amounts_in);

        mint_arrays.ids[index] = id.into();
        mint_arrays.amounts[index] = amounts_in_to_bin;
        mint_arrays.liquidity_minted[index] = shares;

        let amount = shares.u256_to_uint256();

        //Minting tokens
        mint_tokens.push(TokenAmount {
            token_id: id.to_string(),
            balances: vec![TokenIdBalance {
                address: to.clone(),
                amount,
            }],
        });
    }
    let msg = lb_token::ExecuteMsg::MintTokens {
        mint_tokens,
        memo: None,
        padding: None,
    }
    .to_cosmos_msg(
        config.lb_token.code_hash.clone(),
        config.lb_token.address.to_string(),
        None,
    )?;

    messages.push(msg);
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
    time: u64,
    bin_step: u16,
    active_id: u32,
    id: u32,
    amounts_in: Bytes32,
    mut parameters: PairParameters,
) -> Result<(U256, Bytes32, Bytes32)> {
    let bin_reserves = BIN_MAP.load(deps.storage, id).unwrap_or([0u8; 32]);
    let config = STATE.load(deps.storage)?;
    let price = PriceHelper::get_price_from_id(id, bin_step)?;
    let total_supply = _query_total_supply(
        deps.as_ref(),
        id,
        config.lb_token.code_hash,
        config.lb_token.address,
    )?;
    let (shares, amounts_in) = BinHelper::get_shares_and_effective_amounts_in(
        bin_reserves,
        amounts_in,
        price,
        total_supply,
    )?;

    let amounts_in_to_bin = amounts_in;

    if id == active_id {
        parameters.update_volatility_parameters(id, time)?;

        // Helps calculate fee if there's an implict swap.
        let fees = BinHelper::get_composition_fees(
            bin_reserves,
            parameters,
            bin_step,
            amounts_in,
            total_supply,
            shares,
        )?;

        if fees != [0u8; 32] {
            let user_liquidity = BinHelper::get_liquidity(amounts_in.sub(fees), price)?;
            let bin_liquidity = BinHelper::get_liquidity(bin_reserves, price)?;

            let _shares =
                U256x256Math::mul_div_round_down(user_liquidity, total_supply, bin_liquidity)?;
            let protocol_c_fees =
                fees.scalar_mul_div_basis_point_round_down(parameters.get_protocol_share().into())?;

            if protocol_c_fees != [0u8; 32] {
                let _amounts_in_to_bin = amounts_in_to_bin.sub(protocol_c_fees);
                STATE.update(deps.storage, |mut state| -> StdResult<_> {
                    state.protocol_fees = state.protocol_fees.add(protocol_c_fees);
                    Ok(state)
                })?;
            }

            let oracle_id = parameters.get_oracle_id();

            // let mut oracle = ORACLE.load(deps.storage, oracle_id)?;
            // let new_sample;
            // (parameters, new_sample) =
            //     oracle.update(time.seconds(), parameters, id, DEFAULT_ORACLE_LENGTH)?;
            // if let Some(n_s) = new_sample {
            //     ORACLE.save(deps.storage, parameters.get_oracle_id(), &Oracle(n_s))?;
            // }

            parameters = ORACLE.update_oracle(deps.storage, time, parameters, id)?;

            STATE.update(deps.storage, |mut state| -> StdResult<_> {
                state.pair_parameters = parameters;
                Ok(state)
            })?;
        }
    } else {
        BinHelper::verify_amounts(amounts_in, active_id, id)?;
    }

    if shares == 0 || amounts_in_to_bin == [0u8; 32] {
        return Err(Error::ZeroAmount { id });
    }

    if total_supply == 0 {
        BIN_TREE.update(deps.storage, |mut tree| -> StdResult<_> {
            tree.add(id);
            Ok(tree)
        })?;
    }

    BIN_MAP.save(deps.storage, id, &bin_reserves.add(amounts_in_to_bin))?;

    Ok((shares, amounts_in, amounts_in_to_bin))
}

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
    env: Env,
    info: MessageInfo,
    from: String,
    to: String,
    ids: Vec<u32>,
    amounts_to_burn: Vec<Uint256>,
) -> Result<Response> {
    let mut config = STATE.load(deps.storage)?;

    let token_x = config.token_x;
    let token_y = config.token_y;

    if ids.is_empty() || ids.len() != amounts_to_burn.len() {
        return Err(Error::InvalidInput);
    }

    let mut messages: Vec<CosmosMsg> = Vec::new();
    let mut burn_tokens: Vec<TokenAmount> = Vec::new();

    let mut amounts = vec![[0u8; 32]; ids.len()];
    let mut amounts_out = [0u8; 32];

    for i in 0..ids.len() {
        let id = ids[i];
        let amount_to_burn = amounts_to_burn[i];

        if amount_to_burn.is_zero() {
            return Err(Error::ZeroShares { id });
        }

        let bin_reserves = BIN_MAP
            .load(deps.storage, id)
            .map_err(|_| Error::ZeroBinReserve {
                active_id: i as u32,
            })?;
        let total_supply = _query_total_supply(
            deps.as_ref(),
            id,
            config.lb_token.code_hash.clone(),
            config.lb_token.address.clone(),
        )?;

        burn_tokens.push(TokenAmount {
            token_id: id.to_string(),
            balances: vec![TokenIdBalance {
                address: info.sender.clone(),
                amount: amount_to_burn,
            }],
        });

        let amount_to_burn_u256 = amount_to_burn.uint256_to_u256();

        let amounts_out_from_bin_vals =
            BinHelper::get_amount_out_of_bin(bin_reserves, amount_to_burn_u256, total_supply)?;
        let amounts_out_from_bin: Bytes32 =
            Bytes32::encode(amounts_out_from_bin_vals.0, amounts_out_from_bin_vals.1);

        if amounts_out_from_bin.iter().all(|&x| x == 0) {
            return Err(Error::ZeroAmountsOut {
                id,
                amount_to_burn: amount_to_burn_u256.u256_to_uint256(),
                total_supply: total_supply.u256_to_uint256(),
            });
        }

        let bin_reserves = bin_reserves.sub(amounts_out_from_bin);

        if total_supply == amount_to_burn_u256 {
            BIN_MAP.remove(deps.storage, id);
            BIN_TREE.update(deps.storage, |mut tree| -> StdResult<_> {
                tree.remove(id);
                Ok(tree)
            })?;
        } else {
            BIN_MAP.save(deps.storage, id, &bin_reserves)?;
        }

        amounts[i] = amounts_out_from_bin;
        amounts_out = amounts_out.add(amounts_out_from_bin);
    }

    let msg = lb_token::ExecuteMsg::BurnTokens {
        burn_tokens,
        memo: None,
        padding: None,
    }
    .to_cosmos_msg(
        config.lb_token.code_hash,
        config.lb_token.address.to_string(),
        None,
    )?;

    messages.push(msg);

    let raw_msgs = bin_transfer(amounts_out, token_x, token_y, info.sender);

    // TODO: is it better to use `update` here or `save`?
    STATE.update(deps.storage, |mut state| -> StdResult<State> {
        state.reserves = state.reserves.sub(amounts_out);
        Ok(state)
    })?;

    // config.reserves = config.reserves.sub(amounts_out);
    // STATE.save(deps.storage, &config)?;

    if let Some(msgs) = raw_msgs {
        messages.extend(msgs)
    }

    let response_data = to_binary(&BurnResponse { amounts })?;

    Ok(Response::default()
        .set_data(response_data)
        .add_messages(messages))

    // the burn and transfer messages are "fire and forget"
    // the amounts burned go back to the router contract that called this execute message
}

// Administrative functions

/// Collect the protocol fees from the pool.
pub fn collect_protocol_fees(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    // only_protocol_fee_recipient(&info.sender, &state.factory.address)?;

    let token_x = state.token_x;
    let token_y = state.token_y;

    let mut messages: Vec<CosmosMsg> = Vec::new();

    let protocol_fees = state.protocol_fees;

    let (x, y) = protocol_fees.decode();
    let ones = Bytes32::encode(if x > 0 { 1 } else { 0 }, if y > 0 { 1 } else { 0 });

    //The purpose of subtracting ones from the protocolFees is to leave a small amount (1 unit of each token) in the protocol fees.
    //This is done to avoid completely draining the fees and possibly causing any issues with calculations that depend on non-zero values
    let collected_protocol_fees = protocol_fees.sub(ones);

    if U256::from_le_bytes(collected_protocol_fees) != U256::ZERO {
        // This is setting the protocol fees to the smallest possible values
        STATE.update(deps.storage, |mut state| -> StdResult<State> {
            state.protocol_fees = ones;
            state.reserves = state.reserves.sub(collected_protocol_fees);
            Ok(state)
        })?;

        if collected_protocol_fees.iter().any(|&x| x != 0) {
            if let Some(msgs) = bin_transfer(
                collected_protocol_fees,
                token_x.clone(),
                token_y.clone(),
                state.protocol_fees_recipient,
            ) {
                messages.extend(msgs);
            };
        }

        Ok(Response::default()
            .add_attribute(
                format!("Collected Protocol Fees for token {}", token_x.unique_key()),
                collected_protocol_fees.decode_x().to_string(),
            )
            .add_attribute(
                format!("Collected Protocol Fees for token {}", token_y.unique_key()),
                collected_protocol_fees.decode_y().to_string(),
            )
            .add_attribute("Action performed by", info.sender.to_string())
            .add_messages(messages))
    } else {
        Err(Error::NotEnoughFunds)
    }
}

/// Increase the length of the oracle used by the pool.
///
/// # Arguments
///
/// * `new_length` - The new length of the oracle
pub fn increase_oracle_length(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_length: u16,
) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    let mut params = state.pair_parameters;

    let mut oracle_id = params.get_oracle_id();

    // activate the oracle if it is not active yet
    if oracle_id == 0 {
        oracle_id = 1;
        params.set_oracle_id(oracle_id);
    }

    ORACLE.increase_length(deps.storage, oracle_id, new_length)?;

    Ok(Response::default()
        .add_attribute_plaintext("Oracle Length Increased", new_length.to_string()))
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
    let state = STATE.load(deps.storage)?;
    only_factory(&info.sender, &state.factory.address)?;

    let mut params = state.pair_parameters;

    params.set_static_fee_parameters(
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    )?;

    let total_fee = params.get_base_fee(state.bin_step) + params.get_variable_fee(state.bin_step);
    if total_fee > MAX_FEE {
        return Err(Error::MaxTotalFeeExceeded {});
    }

    STATE.update(deps.storage, |mut state| -> StdResult<State> {
        state.pair_parameters = params;
        Ok(state)
    })?;

    Ok(Response::default().add_attribute("status", "ok"))
}

/// Forces the decay of the volatility reference variables.
///
/// Can only be called by the factory.
pub fn force_decay(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    only_factory(&info.sender, &state.factory.address)?;

    let mut params = state.pair_parameters;
    params.update_id_reference();
    params.update_volatility_reference()?;

    STATE.update(deps.storage, |mut state| -> StdResult<State> {
        state.pair_parameters = params;
        Ok(state)
    })?;

    Ok(Response::default())
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
