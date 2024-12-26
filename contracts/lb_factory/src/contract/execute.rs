use super::{
    helper::{_is_preset_open, _sort_tokens},
    state::*,
    CREATE_LB_PAIR_REPLY_ID, MIN_BIN_STEP, OFFSET_IS_PRESET_OPEN,
};
use crate::prelude::*;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, Event, MessageInfo, Response, SubMsg, WasmMsg,
};
use lb_interfaces::{
    lb_factory::*,
    lb_pair::{ExecuteMsg as LbPairExecuteMsg, InstantiateMsg as LbPairInstantiateMsg, LbPair},
};
use lb_libraries::{
    math::encoded::Encoded, pair_parameter_helper::PairParameters, price_helper::PriceHelper,
};
use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    swap::core::TokenType,
    utils::callback::ExecuteCallback,
};

/// Sets the LbPair implementation details.
///
/// # Arguments
///
/// * `new_lb_pair_implementation` - The code ID and code hash of the implementation.
pub fn set_lb_pair_implementation(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_lb_pair_implementation: Implementation,
) -> Result<Response> {
    let config = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    let old_lb_pair_implementation = LB_PAIR_IMPLEMENTATION.load(deps.storage)?;
    if old_lb_pair_implementation == new_lb_pair_implementation {
        return Err(Error::SameImplementation {
            implementation: old_lb_pair_implementation.id,
        });
    }

    LB_PAIR_IMPLEMENTATION.save(deps.storage, &new_lb_pair_implementation)?;

    let event = Event::lb_pair_implementation_set(
        old_lb_pair_implementation.id,
        new_lb_pair_implementation.id,
    );

    Ok(Response::new().add_event(event))
}

/// Sets the LbToken implementation details.
///
/// # Arguments
///
/// * `new_lb_token_implementation` - The code ID and code hash of the implementation.
pub fn set_lb_token_implementation(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_lb_token_implementation: Implementation,
) -> Result<Response> {
    let config = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    let old_lb_token_implementation = LB_TOKEN_IMPLEMENTATION.load(deps.storage)?;
    if old_lb_token_implementation == new_lb_token_implementation {
        return Err(Error::SameImplementation {
            implementation: old_lb_token_implementation.id,
        });
    }

    LB_TOKEN_IMPLEMENTATION.save(deps.storage, &new_lb_token_implementation)?;

    let event = Event::lb_token_implementation_set(
        old_lb_token_implementation.id,
        new_lb_token_implementation.id,
    );

    Ok(Response::new().add_event(event))
}

/// Creates a liquidity bin LbPair for token_x and token_y.
///
/// # Arguments
///
/// * `token_x` - The address of the first token.
/// * `token_y` - The address of the second token.
/// * `active_id` - The active id of the pair.
/// * `bin_step` - The bin step in basis point, used to calculate log(1 + binStep / 10_000).
///
/// # Returns
///
/// * `pair` - The address of the newly created LbPair.
pub fn create_lb_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_x: TokenType,
    token_y: TokenType,
    active_id: u32,
    bin_step: u16,
    viewing_key: String,
    entropy: String,
) -> Result<Response> {
    let config = STATE.load(deps.storage)?;

    // TODO: I think this is redundant
    // if !PRESETS.contains(deps.storage, &bin_step) {
    //     return Err(Error::BinStepHasNoPreset { bin_step });
    // }

    let preset = PRESETS
        .get(deps.storage, &bin_step)
        .ok_or_else(|| Error::BinStepHasNoPreset { bin_step })?;

    let is_owner = info.sender == config.owner;

    if !_is_preset_open(preset.0) && !is_owner {
        return Err(Error::PresetIsLockedForUsers {
            user: info.sender,
            bin_step,
        });
    }

    if !QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .any(|result| match result {
            Ok(t) => t.eq(&token_y),
            Err(_) => false,
        })
    {
        return Err(Error::QuoteAssetNotWhitelisted {
            quote_asset: token_y.unique_key(),
        });
    }

    if token_x == token_y {
        return Err(Error::IdenticalAddresses {
            token: token_x.unique_key(),
        });
    }

    // safety check, making sure that the price can be calculated
    PriceHelper::get_price_from_id(active_id, bin_step)?;

    // We sort token for storage efficiency, only one input needs to be stored because they are sorted
    let (token_a, token_b) = _sort_tokens(token_x.clone(), token_y.clone());

    if LB_PAIRS_INFO
        .get(
            deps.storage,
            &(token_a.unique_key(), token_b.unique_key(), bin_step),
        )
        .is_some()
    {
        return Err(Error::LbPairAlreadyExists {
            token_x: token_x.unique_key(),
            token_y: token_y.unique_key(),
            bin_step,
        });
    }

    let lb_pair_implementation = LB_PAIR_IMPLEMENTATION.load(deps.storage)?;
    let lb_token_implementation = LB_TOKEN_IMPLEMENTATION.load(deps.storage)?;

    if lb_pair_implementation.id == 0 {
        return Err(Error::ImplementationNotSet);
    }

    let mut messages = vec![];

    messages.push(SubMsg::reply_on_success(
        WasmMsg::Instantiate {
            code_id: lb_pair_implementation.id,
            label: format!(
                "{}-{}-{}-pair-{}-{}",
                token_x.unique_key(),
                token_y.unique_key(),
                bin_step,
                env.contract.address,
                lb_pair_implementation.id,
            ),
            msg: to_binary(&LbPairInstantiateMsg {
                factory: env.contract,
                token_x: token_x.clone(),
                token_y: token_y.clone(),
                bin_step,
                pair_parameters: StaticFeeParameters {
                    base_factor: preset.get_base_factor(),
                    filter_period: preset.get_filter_period(),
                    decay_period: preset.get_decay_period(),
                    reduction_factor: preset.get_reduction_factor(),
                    variable_fee_control: preset.get_variable_fee_control(),
                    protocol_share: preset.get_protocol_share(),
                    max_volatility_accumulator: preset.get_max_volatility_accumulator(),
                },
                active_id,
                lb_token_implementation,
                viewing_key,
                entropy,
                admin_auth: config.admin_auth.into(),
                query_auth: config.query_auth.into(),
            })?,
            code_hash: lb_pair_implementation.code_hash.clone(),
            funds: vec![],
            admin: None,
        },
        CREATE_LB_PAIR_REPLY_ID,
    ));

    EPHEMERAL_LB_PAIR.save(
        deps.storage,
        &EphemeralLbPair {
            token_x,
            token_y,
            bin_step,
            code_hash: lb_pair_implementation.code_hash,
            created_by_owner: is_owner,
        },
    )?;

    Ok(Response::new().add_submessages(messages))
}

pub fn set_lb_pair_ignored(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
    ignored: bool,
) -> Result<Response> {
    todo!()

    // let event = Event::lb_pair_ignored_state_changed(lb_pair, ignored);
    //
    // Ok(Response::new().add_event(event))
}

/// Sets the preset parameters of a bin step
///
/// # Arguments
///
/// * `bin_step` - The bin step in basis point, used to calculate the price
/// * `base_factor` - The base factor, used to calculate the base fee, baseFee = baseFactor * binStep
/// * `filter_period` - The period where the accumulator value is untouched, prevent spam
/// * `decay_period` - The period where the accumulator value is decayed, by the reduction factor
/// * `reduction_factor` - The reduction factor, used to calculate the reduction of the accumulator
/// * `variable_fee_control` - The variable fee control, used to control the variable fee, can be 0 to disable it
/// * `protocol_share` - The share of the fees received by the protocol
/// * `max_volatility_accumulator` - The max value of the volatility accumulator
/// * `is_open` - Whether the preset is open or not to be used by users
pub fn set_pair_preset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bin_step: u16,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
    is_open: bool,
) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    if bin_step < MIN_BIN_STEP as u16 {
        return Err(Error::BinStepTooLow { bin_step });
    }

    let mut preset = PairParameters::default();

    preset.set_static_fee_parameters(
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    )?;

    if is_open {
        preset.0.set_bool(true, OFFSET_IS_PRESET_OPEN);
    }

    PRESET_BIN_STEPS.insert(deps.storage, &bin_step)?;

    PRESETS.insert(deps.storage, &bin_step, &preset)?;
    STATE.save(deps.storage, &state)?;

    let event = Event::preset_set(
        bin_step,
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

/// Sets if the preset is open or not to be used by users
///
/// # Arguments
///
/// * `bin_step` - The bin step in basis point, used to calculate the price
/// * `is_open` - Whether the preset is open or not
pub fn set_preset_open_state(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bin_step: u16,
    is_open: bool,
) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;

    let Some(mut preset) = PRESETS.get(deps.storage, &bin_step) else {
        return Err(Error::BinStepHasNoPreset { bin_step });
    };

    if preset.0.decode_bool(OFFSET_IS_PRESET_OPEN) == is_open {
        return Err(Error::PresetOpenStateIsAlreadyInTheSameState);
    } else {
        preset.0.set_bool(is_open, OFFSET_IS_PRESET_OPEN);
    }

    PRESETS.insert(deps.storage, &bin_step, &preset)?;

    let event = Event::preset_open_state_changed(bin_step, is_open);

    Ok(Response::new().add_event(event))
}

/// Remove the preset linked to a bin_step
///
/// # Arguments
///
/// * `bin_step` - The bin step to remove
pub fn remove_preset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bin_step: u16,
) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;

    if !PRESETS.contains(deps.storage, &bin_step) {
        return Err(Error::BinStepHasNoPreset { bin_step });
    }

    PRESETS.remove(deps.storage, &bin_step)?;
    PRESET_BIN_STEPS.remove(deps.storage, &bin_step)?;

    let event = Event::preset_removed(bin_step);

    Ok(Response::new().add_event(event))
}

/// Function to set the fee parameters of a LbPair
///
/// # Arguments
///
/// * `token_x` - The address of the first token
/// * `token_y` - The address of the second token
/// * `bin_step` - The bin step in basis point, used to calculate the price
/// * `base_factor` - The base factor, used to calculate the base fee, baseFee = baseFactor * binStep
/// * `filter_period` - The period where the accumulator value is untouched, prevent spam
/// * `decay_period` - The period where the accumulator value is decayed, by the reduction factor
/// * `reduction_factor` - The reduction factor, used to calculate the reduction of the accumulator
/// * `variable_fee_control` - The variable fee control, used to control the variable fee, can be 0 to disable it
/// * `protocol_share` - The share of the fees received by the protocol
/// * `max_volatility_accumulator` - The max value of volatility accumulator
pub fn set_fee_parameters_on_pair(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
) -> Result<Response> {
    let state = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &state.admin_auth,
    )?;
    let (token_a, token_b) = _sort_tokens(token_x, token_y);
    let lb_pair = LB_PAIRS_INFO
        .get(
            deps.storage,
            &(token_a.unique_key(), token_b.unique_key(), bin_step),
        )
        .ok_or_else(|| Error::LbPairNotCreated {
            token_x: token_a.unique_key(),
            token_y: token_b.unique_key(),
            bin_step,
        })?
        .lb_pair;

    let msg: CosmosMsg = LbPairExecuteMsg::SetStaticFeeParameters {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    }
    .to_cosmos_msg(&lb_pair.contract, vec![])?;

    Ok(Response::new().add_message(msg))
}

/// Function to set the recipient of the fees. This address needs to be able to receive SNIP20s.
///
/// # Arguments
///
/// * `fee_recipient` - The address of the recipient
pub fn set_fee_recipient(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fee_recipient: Addr,
) -> Result<Response> {
    let config = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    let old_fee_recipient = FEE_RECIPIENT.load(deps.storage)?;
    if old_fee_recipient == fee_recipient {
        return Err(Error::SameFeeRecipient {
            fee_recipient: old_fee_recipient,
        });
    }

    FEE_RECIPIENT.save(deps.storage, &fee_recipient)?;

    let event = Event::fee_recipient_set(old_fee_recipient, fee_recipient);

    Ok(Response::new().add_event(event))
}

/// Function to add an asset to the whitelist of quote assets
///
/// # Arguments
///
/// * `quote_asset` - The quote asset (e.g: NATIVE, USDC...)
pub fn add_quote_asset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    quote_asset: TokenType,
) -> Result<Response> {
    let config = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;
    if QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .any(|result| match result {
            Ok(t) => t.eq(&quote_asset),
            Err(_) => false, // Handle the error case as needed
        })
    {
        return Err(Error::QuoteAssetAlreadyWhitelisted {
            quote_asset: quote_asset.unique_key(),
        });
    }

    QUOTE_ASSET_WHITELIST.push(deps.storage, &quote_asset)?;

    let event = Event::quote_asset_added(quote_asset.unique_key());

    Ok(Response::new().add_event(event))
}

/// Function to remove an asset from the whitelist of quote assets
///
/// # Arguments
///
/// * `quote_asset` - The quote asset (e.g: NATIVE, USDC...)
pub fn remove_quote_asset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: TokenType,
) -> Result<Response> {
    let config = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;
    // TODO: there has to be a better way to write this
    let found_asset = QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .enumerate()
        .find(|(_, result)| result.as_ref().ok().map_or(false, |t| t.eq(&asset)));

    match found_asset {
        Some((index, Ok(quote_asset))) => {
            QUOTE_ASSET_WHITELIST.remove(deps.storage, index as u32)?;

            let event = Event::quote_asset_removed(quote_asset.unique_key());

            Ok(Response::new().add_event(event))
        }
        _ => {
            return Err(Error::QuoteAssetNotWhitelisted {
                quote_asset: asset.unique_key(),
            });
        }
    }
}

pub fn force_decay(deps: DepsMut, _env: Env, info: MessageInfo, pair: LbPair) -> Result<Response> {
    let config = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::LiquidityBookAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    let (token_a, token_b) = _sort_tokens(pair.token_x, pair.token_y);
    let lb_pair = LB_PAIRS_INFO
        .get(
            deps.storage,
            &(token_a.unique_key(), token_b.unique_key(), pair.bin_step),
        )
        .ok_or_else(|| Error::LbPairNotCreated {
            token_x: token_a.unique_key(),
            token_y: token_b.unique_key(),
            bin_step: pair.bin_step,
        })?
        .lb_pair;

    let mut response = Response::new();

    response = response.add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: lb_pair.contract.address.to_string(),
        code_hash: lb_pair.contract.code_hash,
        msg: to_binary(&LbPairExecuteMsg::ForceDecay {})?,
        funds: vec![],
    }));

    Ok(response)
}
