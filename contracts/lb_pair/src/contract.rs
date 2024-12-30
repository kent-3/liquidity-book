use crate::{execute::*, helper::*, prelude::*, query::*, state::*};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, ContractInfo, CosmosMsg, Deps, DepsMut, Env,
    Event, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, SubMsgResult, Uint128,
    WasmMsg,
};
use liquidity_book::{
    interfaces::{lb_pair::*, lb_token, lb_token::state_structs::LbPair},
    libraries::{constants, BinHelper, Bytes32, PackedUint128Math, PairParameters},
};
// TODO: get rid of admin stuff and shade_protocol dependency
use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    swap::core::{TokenType, ViewingKey},
};

pub const INSTANTIATE_LB_TOKEN_REPLY_ID: u64 = 1u64;
pub const FLASH_LOAN_REPLY_ID: u64 = 999u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response> {
    const LB_TOKEN_DECIMALS: u8 = 18;
    // TODO: isn't this supposed to start at 0?
    // const START_ORACLE_ID: u16 = 1;

    // Initializing the Token Contract
    let token_x_symbol = match msg.token_x.clone() {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => query_token_symbol(deps.as_ref(), token_code_hash, contract_addr)?,
        TokenType::NativeToken { denom } => denom,
    };

    let token_y_symbol = match msg.token_y.clone() {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => query_token_symbol(deps.as_ref(), token_code_hash, contract_addr)?,
        TokenType::NativeToken { denom } => denom,
    };

    let instantiate_token_msg = lb_token::InstantiateMsg {
        has_admin: false,
        admin: None,
        curators: [env.contract.address.clone()].to_vec(),
        entropy: msg.entropy,
        lb_pair_info: LbPair {
            name: format!(
                "Lb-token-{}-{}-{}",
                token_x_symbol, token_y_symbol, &msg.bin_step
            ),
            symbol: format!("LB-{}-{}-{}", token_x_symbol, token_y_symbol, &msg.bin_step),
            lb_pair_address: env.contract.address.clone(),
            decimals: LB_TOKEN_DECIMALS,
        },
        initial_tokens: Vec::new(),
    };

    let mut response = Response::new();
    response = response.add_submessage(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: msg.lb_token_implementation.id,
            code_hash: msg.lb_token_implementation.code_hash.clone(),
            msg: to_binary(&instantiate_token_msg)?,
            label: format!(
                "{}-{}-Pair-Token-{}-{}",
                token_x_symbol, token_y_symbol, msg.bin_step, env.block.height
            ),
            funds: vec![],
            admin: None,
        }),
        INSTANTIATE_LB_TOKEN_REPLY_ID,
    ));

    let mut pair_parameters = PairParameters::default();
    pair_parameters.set_static_fee_parameters(
        msg.pair_parameters.base_factor,
        msg.pair_parameters.filter_period,
        msg.pair_parameters.decay_period,
        msg.pair_parameters.reduction_factor,
        msg.pair_parameters.variable_fee_control,
        msg.pair_parameters.protocol_share,
        msg.pair_parameters.max_volatility_accumulator,
    )?;
    pair_parameters.set_active_id(msg.active_id)?;
    // pair_parameters.set_oracle_id(START_ORACLE_ID); // Activating the oracle
    pair_parameters.update_id_reference();

    // RegisterReceiving Token
    let mut messages = vec![];
    let viewing_key = ViewingKey::from(msg.viewing_key.as_str());
    for token in [&msg.token_x, &msg.token_y] {
        if let TokenType::CustomToken { .. } = token {
            register_pair_token(&env, &mut messages, token, &viewing_key)?;
        }
    }

    let state = State {
        creator: info.sender,
        // viewing_key,
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
    };

    // TODO: rename?
    STATE.save(deps.storage, &state)?;
    CONTRACT_STATUS.save(deps.storage, &ContractStatus::Active)?;
    VIEWING_KEY.save(deps.storage, &viewing_key)?;

    TOKEN_X.save(deps.storage, &msg.token_x)?;
    TOKEN_Y.save(deps.storage, &msg.token_y)?;
    BIN_STEP.save(deps.storage, &msg.bin_step)?;

    PARAMETERS.save(deps.storage, &pair_parameters)?;
    RESERVES.save(deps.storage, &Bytes32::default())?;
    PROTOCOL_FEES.save(deps.storage, &Bytes32::default())?;

    LB_TOKEN.save(
        deps.storage,
        &ContractInfo {
            address: Addr::unchecked(""),
            code_hash: "".to_string(),
        },
    )?;

    EPHEMERAL_LB_TOKEN.save(
        deps.storage,
        &EphemeralLbToken {
            code_hash: msg.lb_token_implementation.code_hash,
        },
    )?;

    response = response.add_messages(messages);
    response = response.set_data(env.contract.address.as_bytes());

    Ok(response)
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    match CONTRACT_STATUS.load(deps.storage)? {
        ContractStatus::FreezeAll => match msg {
            ExecuteMsg::Mint { .. }
            | ExecuteMsg::Swap { .. }
            | ExecuteMsg::Burn { .. }
            | ExecuteMsg::Receive(..) => {
                return Err(Error::TransactionBlock());
            }
            _ => {}
        },
        ContractStatus::LpWithdrawOnly => match msg {
            ExecuteMsg::Mint { .. } | ExecuteMsg::Swap { .. } => {
                return Err(Error::TransactionBlock());
            }
            _ => {}
        },
        ContractStatus::Active => {}
    }

    match msg {
        ExecuteMsg::Swap { swap_for_y, to } => swap(deps, env, info, swap_for_y, to),
        // TODO:
        ExecuteMsg::FlashLoan {
            receiver,
            amounts,
            data,
        } => flash_loan(deps, env, info, receiver, amounts, data),
        ExecuteMsg::Mint {
            to,
            liquidity_configs,
            refund_to,
        } => mint(deps, env, info, to, liquidity_configs, refund_to),
        ExecuteMsg::Burn {
            from,
            to,
            ids,
            amounts_to_burn,
        } => burn(deps, env, info, from, to, ids, amounts_to_burn),
        ExecuteMsg::CollectProtocolFees {} => collect_protocol_fees(deps, env, info),
        ExecuteMsg::IncreaseOracleLength { new_length } => {
            increase_oracle_length(deps, env, info, new_length)
        }
        ExecuteMsg::SetStaticFeeParameters {
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        } => set_static_fee_parameters(
            deps,
            env,
            info,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        ),
        ExecuteMsg::SetHooksParameters {
            hooks_parameters,
            on_hooks_set_data,
        } => todo!(),
        ExecuteMsg::ForceDecay {} => {
            // TODO: this is kinda neat, but I think it's better to keep it inside the function
            only_factory(&info.sender, &FACTORY.load(deps.storage)?.address)?;
            force_decay(deps, env, info)
        }
        ExecuteMsg::BatchTransferFrom {
            from,
            to,
            ids,
            amounts,
        } => batch_transfer_from(deps, env, info, from, to, ids, amounts),

        // not in joe-v2
        ExecuteMsg::SetContractStatus { contract_status } => {
            let state = STATE.load(deps.storage)?;
            validate_admin(
                &deps.querier,
                AdminPermissions::ShadeSwapAdmin,
                &info.sender,
                &state.admin_auth,
            )?;
            CONTRACT_STATUS.save(deps.storage, &contract_status)?;

            Ok(Response::default().add_attribute("new_status", contract_status.to_string()))
        }
        ExecuteMsg::Receive(msg) => {
            let checked_addr = deps.api.addr_validate(&msg.from)?;
            receiver_callback(deps, env, info, checked_addr, msg.amount, msg.msg)
        }
    }
}

// TODO: I don't think we need this! The swap function will always be called by the lb-router, who
// has the ability to transfer tokens to the lb-pair before-hand. A user sends tokens to the
// router, so I think we have some refactoring to do over there to handle messages through the
// receiver_callback.

pub fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> Result<Response> {
    let msg = msg.ok_or(Error::ReceiverMsgEmpty)?;

    let response = match from_binary(&msg)? {
        InvokeMsg::Swap { swap_for_y, to } => {
            // this check needs to be here instead of in execute() because it is impossible to (cleanly) distinguish between swaps and lp withdraws until this point
            // if contract_status is FreezeAll, this fn will never be called, so only need to check LpWithdrawOnly here
            if CONTRACT_STATUS.load(deps.storage)? == ContractStatus::LpWithdrawOnly {
                return Err(Error::TransactionBlock());
            }

            if info.sender != TOKEN_X.load(deps.storage)?.unique_key()
                && info.sender != TOKEN_Y.load(deps.storage)?.unique_key()
            {
                return Err(Error::NoMatchingTokenInPair);
            }

            swap(deps, env, info, swap_for_y, to)?
        }
    };
    Ok(response)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetFactory {} => to_binary(&query_factory(deps)?),
        QueryMsg::GetTokenX {} => to_binary(&query_token_x(deps)?),
        QueryMsg::GetTokenY {} => to_binary(&query_token_y(deps)?),
        QueryMsg::GetBinStep {} => to_binary(&query_bin_step(deps)?),
        QueryMsg::GetReserves {} => to_binary(&query_reserves(deps)?),
        QueryMsg::GetActiveId {} => to_binary(&query_active_id(deps)?),
        QueryMsg::GetBin { id } => to_binary(&query_bin(deps, id)?),
        QueryMsg::GetNextNonEmptyBin { swap_for_y, id } => {
            to_binary(&query_next_non_empty_bin(deps, swap_for_y, id)?)
        }
        QueryMsg::GetProtocolFees {} => to_binary(&query_protocol_fees(deps)?),
        QueryMsg::GetStaticFeeParameters {} => to_binary(&query_static_fee_parameters(deps)?),
        QueryMsg::GetLbHooksParameters {} => todo!(),
        QueryMsg::GetVariableFeeParameters {} => to_binary(&query_variable_fee_parameters(deps)?),
        QueryMsg::GetOracleParameters {} => to_binary(&query_oracle_params(deps)?),
        QueryMsg::GetOracleSampleAt { lookup_timestamp } => {
            to_binary(&query_oracle_sample_at(deps, env, lookup_timestamp)?)
        }
        QueryMsg::GetPriceFromId { id } => to_binary(&query_price_from_id(deps, id)?),
        QueryMsg::GetIdFromPrice { price } => to_binary(&query_id_from_price(deps, price)?),
        QueryMsg::GetSwapIn {
            amount_out,
            swap_for_y,
        } => to_binary(&query_swap_in(deps, env, amount_out.u128(), swap_for_y)?),
        QueryMsg::GetSwapOut {
            amount_in,
            swap_for_y,
        } => to_binary(&query_swap_out(deps, env, amount_in.u128(), swap_for_y)?),

        // not in joe-v2
        QueryMsg::GetLbToken {} => to_binary(&query_lb_token(deps)?),
        QueryMsg::GetLbTokenSupply { id } => to_binary(&query_total_supply(deps, id)?),
        QueryMsg::GetBins { ids } => to_binary(&query_bins(deps, ids)?),
        QueryMsg::GetAllBins {
            id,
            page,
            page_size,
        } => to_binary(&query_all_bins(deps, env, page, page_size, id)?),
    }
    .map_err(Error::CwErr)
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response> {
    match (msg.id, msg.result) {
        (INSTANTIATE_LB_TOKEN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                // TODO: decide which way I like best
                // Is the string returned a JSON encoded string?

                // let contract_address_string = &String::from_utf8(x.to_vec())?;
                // let trimmed_str = contract_address_string.trim_matches('\"');
                // let address = deps.api.addr_validate(trimmed_str)?;

                // let address: String = from_binary(&x)?;
                // let address: Addr = deps.api.addr_validate(&address)?;

                // let address = deps.api.addr_validate(std::str::from_utf8(&x)?)?;

                let address = deps.api.addr_validate(&from_binary::<String>(&x)?)?;
                let code_hash = EPHEMERAL_LB_TOKEN.load(deps.storage)?.code_hash;

                LB_TOKEN.save(deps.storage, &ContractInfo { address, code_hash })?;

                let response =
                    Response::new().set_data(env.contract.address.to_string().as_bytes());

                Ok(response)
            }
            None => Err(Error::ReplyDataMissing),
        },
        (FLASH_LOAN_REPLY_ID, SubMsgResult::Err(_error_message)) => {
            // TODO: should we include the receiver contract's error message in ours?
            Err(Error::FlashLoanCallbackFailed)
        }
        (FLASH_LOAN_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(r_data) => {
                if r_data != constants::CALLBACK_SUCCESS {
                    return Err(Error::FlashLoanCallbackFailed);
                }

                let token_x = TOKEN_X.load(deps.storage)?;
                let token_y = TOKEN_Y.load(deps.storage)?;
                let viewing_key = VIEWING_KEY.load(deps.storage)?;

                let token_x_balance = token_x.query_balance(
                    deps.as_ref(),
                    env.contract.address.to_string(),
                    viewing_key.to_string(),
                )?;
                let token_y_balance = token_y.query_balance(
                    deps.as_ref(),
                    env.contract.address.to_string(),
                    viewing_key.to_string(),
                )?;

                // NOTE: This is written to match the original, but in our case it only encodes the token balances.
                let balances_after =
                    [0u8; 32].received(token_x_balance.u128(), token_y_balance.u128());

                let EphemeralFlashLoan {
                    reserves_before,
                    total_fees,
                    sender,
                    receiver,
                    amounts,
                } = EPHEMERAL_FLASH_LOAN.load(deps.storage)?;

                // TODO: check that this explicit type of lt or gt is being used elsewhere
                if PackedUint128Math::lt(&balances_after, reserves_before.add(total_fees)?) {
                    return Err(Error::FlashLoanInsufficientAmount);
                }

                let fees_received = balances_after.sub(reserves_before)?;

                RESERVES.save(deps.storage, &balances_after)?;
                PROTOCOL_FEES.update(deps.storage, |protocol_fees| -> StdResult<_> {
                    protocol_fees
                        .add(fees_received)
                        .map_err(|e| StdError::GenericErr { msg: e.to_string() })
                })?;

                let parameters = PARAMETERS.load(deps.storage)?;

                let event = Event::flash_loan(
                    &sender,
                    &receiver,
                    parameters.get_active_id(),
                    &amounts,
                    &fees_received,
                    &fees_received,
                );

                // TODO: Hooks
                //     Hooks.afterFlashLoan(hooksParameters, msg.sender, address(receiver), totalFees, feesReceived);

                let response = Response::new().add_event(event);

                Ok(response)
            }
            None => Err(Error::ReplyDataMissing),
        },
        _ => Err(Error::UnknownReplyId { id: msg.id }),
    }
}
