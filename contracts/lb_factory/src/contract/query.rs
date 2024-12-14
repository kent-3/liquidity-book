use super::{
    helper::{_get_lb_pair_information, _is_preset_open, _sort_tokens},
    state::*,
    MIN_BIN_STEP, OFFSET_IS_PRESET_OPEN,
};
use crate::prelude::*;
use cosmwasm_std::{to_binary, Binary, Deps};
use lb_interfaces::{lb_factory::*, lb_pair::LbPairInformation};
use lb_libraries::math::encoded::Encoded;
use shade_protocol::swap::core::TokenType;

/// Returns the minimum bin step a pair can have.
///
/// # Returns
///
/// * `min_bin_step` - The minimum bin step of the pair.
pub fn query_min_bin_step(_deps: Deps) -> Result<Binary> {
    let response = MinBinStepResponse {
        min_bin_step: MIN_BIN_STEP,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the protocol fee recipient.
///
/// # Returns
///
/// * `fee_recipient` - The address of the fee recipient.
pub fn query_fee_recipient(deps: Deps) -> Result<Binary> {
    let config = STATE.load(deps.storage)?;
    let response = FeeRecipientResponse {
        fee_recipient: config.fee_recipient,
    };
    to_binary(&response).map_err(Error::CwErr)
}

pub fn query_max_flash_loan_fee(deps: Deps) -> Result<Binary> {
    todo!()
}

pub fn query_flash_loan_fee(deps: Deps) -> Result<Binary> {
    todo!()
}

/// Returns the code ID and hash of the LbPair implementation.
///
/// # Returns
///
/// * `lb_pair_implementation` - The code ID and hash of the LbPair implementation.
pub fn query_lb_pair_implementation(deps: Deps) -> Result<Binary> {
    let config = STATE.load(deps.storage)?;
    let response = LbPairImplementationResponse {
        lb_pair_implementation: config.lb_pair_implementation,
    };
    to_binary(&response).map_err(Error::CwErr)
}

// TODO: this isn't in joe-v2
// Returns the code ID and hash of the LbToken implementation.
///
/// # Returns
///
/// * `lb_token_implementation` - The code ID and hash of the LbToken implementation.
pub fn query_lb_token_implementation(deps: Deps) -> Result<Binary> {
    let config = STATE.load(deps.storage)?;
    let response = LbTokenImplementationResponse {
        lb_token_implementation: config.lb_token_implementation,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the number of LbPairs created.
///
/// # Returns
///
/// * `lb_pair_number` - The number of LbPairs created.
pub fn query_number_of_lb_pairs(deps: Deps) -> Result<Binary> {
    let lb_pair_number = ALL_LB_PAIRS.get_len(deps.storage)?;

    let response = NumberOfLbPairsResponse { lb_pair_number };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the LbPair created at index `index`.
///
/// # Arguments
///
/// * `index` - The index of the LbPair.
///
/// # Returns
///
/// * lb_pair - The address of the LbPair at index `index`.
pub fn query_lb_pair_at_index(deps: Deps, index: u32) -> Result<Binary> {
    let lb_pair = ALL_LB_PAIRS.get_at(deps.storage, index)?;

    let response = LbPairAtIndexResponse { lb_pair };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the number of quote assets whitelisted.
///
/// # Returns
///
/// * `number_of_quote_assets` - The number of quote assets.
pub fn query_number_of_quote_assets(deps: Deps) -> Result<Binary> {
    let number_of_quote_assets = QUOTE_ASSET_WHITELIST.get_len(deps.storage)?;

    let response = NumberOfQuoteAssetsResponse {
        number_of_quote_assets,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the quote asset whitelisted at index `index`.
///
/// # Arguments
///
/// * `index` - The index of the quote asset.
///
/// # Returns
///
/// * `asset` - The address of the quote asset at index `index`.
// TODO: Unsure if this function is necessary. Not sure how to index the Keyset. WAITING: For Front-end to make some decisions about this
pub fn query_quote_asset_at_index(deps: Deps, index: u32) -> Result<Binary> {
    let asset = QUOTE_ASSET_WHITELIST.get_at(deps.storage, index)?;

    let response = QuoteAssetAtIndexResponse { asset };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns whether a token is a quote asset (true) or not (false).
///
/// # Arguments
///
/// * `token` - The address of the asset.
pub fn query_is_quote_asset(deps: Deps, token: TokenType) -> Result<Binary> {
    let is_quote = QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .any(|result| match result {
            Ok(t) => t.eq(&token),
            Err(_) => false,
        });

    let response = IsQuoteAssetResponse { is_quote };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the LbPairInformation if it exists, if not, then the address 0 is returned.
///
/// # Arguments
///
/// * `token_a` - The address of the first token of the pair.
/// * `token_b` - The address of the second token of the pair.
/// * `bin_step` - The bin step of the LbPair.
///
/// # Returns
///
/// * `lb_pair_information` - The LbPairInformation.
pub fn query_lb_pair_information(
    deps: Deps,
    token_a: TokenType,
    token_b: TokenType,
    bin_step: u16,
) -> Result<Binary> {
    let lb_pair_information: LbPairInformation =
        _get_lb_pair_information(deps, token_a, token_b, bin_step)?;

    let response = LbPairInformationResponse {
        lb_pair_information,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the different parameters of the preset.
///
/// # Arguments
///
/// * `bin_step` - The bin step of the preset.
///
/// # Returns
///
/// * `base_factor` - The base factor of the preset.
/// * `filter_period` - The filter period of the preset.
/// * `decay_period` - The decay period of the preset.
/// * `reduction_factor` - The reduction factor of the preset.
/// * `variable_fee_control` - The variable fee control of the preset.
/// * `protocol_share` - The protocol share of the preset.
/// * `max_volatility_accumulator` - The max volatility accumulator of the preset.
/// * `is_open` - Whether the preset is open or not.
pub fn query_preset(deps: Deps, bin_step: u16) -> Result<Binary> {
    if !PRESETS.has(deps.storage, bin_step) {
        return Err(Error::BinStepHasNoPreset { bin_step });
    }

    // NOTE: each preset is an encoded Bytes32.
    // The PairParameters wrapper provides methods to decode specific values.
    let preset = PRESETS.load(deps.storage, bin_step).unwrap();

    let base_factor = preset.get_base_factor();
    let filter_period = preset.get_filter_period();
    let decay_period = preset.get_decay_period();
    let reduction_factor = preset.get_reduction_factor();
    let variable_fee_control = preset.get_variable_fee_control();
    let protocol_share = preset.get_protocol_share();
    let max_volatility_accumulator = preset.get_max_volatility_accumulator();

    let is_open = preset.0.decode_bool(OFFSET_IS_PRESET_OPEN);

    let response = PresetResponse {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
        is_open,
    };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns the list of available bin steps with a preset.
///
/// # Returns
///
/// * `bin_step_with_preset` - The list of bin steps.
pub fn query_all_bin_steps(deps: Deps) -> Result<Binary> {
    // NOTE: iterating over the keys of the PRESETS Keymap will return all available bin_steps
    // not too confident with this implementation...

    let mut bin_step_with_preset = Vec::<u16>::new();

    let hashset = PRESET_HASHSET.load(deps.storage)?;

    // let iterator = PRESETS.range(deps.storage, None, None, Ascending);

    for bin_step in hashset {
        bin_step_with_preset.push(bin_step)
    }

    let response = AllBinStepsResponse {
        bin_step_with_preset,
    };
    to_binary(&response).map_err(Error::CwErr)
}

// this does the same thing as `query_all_bin_steps` but returns only the ones where `is_open` is true
/// Returns the list of open bin steps.
///
/// # Returns
///
/// * `open_bin_step` - The list of open bin steps.
pub fn query_open_bin_steps(deps: Deps) -> Result<Binary> {
    // this way is harder to ready, but maybe more efficient?

    let hashset = PRESET_HASHSET.load(deps.storage)?;

    let mut open_bin_steps = Vec::<u16>::new();

    for bin_step in hashset {
        let preset = PRESETS.load(deps.storage, bin_step)?;

        if _is_preset_open(preset.0) {
            open_bin_steps.push(bin_step)
        }
    }

    let response = OpenBinStepsResponse { open_bin_steps };
    to_binary(&response).map_err(Error::CwErr)
}

/// Returns all the LbPair of a pair of tokens.
///
/// # Arguments
///
/// * `token_x` - The first token of the pair.
/// * `token_y` - The second token of the pair.
///
/// # Returns
///
/// * `lb_pairs_available` - The list of available LbPairs.
pub fn query_all_lb_pairs(deps: Deps, token_x: TokenType, token_y: TokenType) -> Result<Binary> {
    let (token_a, token_b) = _sort_tokens(token_x, token_y);

    // Create a Vec of available bin steps for this pair
    let bin_steps: Vec<u16> = AVAILABLE_LB_PAIR_BIN_STEPS
        .load(deps.storage, (token_a.unique_key(), token_b.unique_key()))
        .map_err(|_| Error::Generic("This token pair is not in the map".to_string()))?;

    // Not sure if this condition is possible, but just in case.
    if bin_steps.is_empty() {
        return Err(Error::Generic("No available bin_steps".to_string()));
    }

    // Collect LbPairInformation values into a vector
    let lb_pairs_available: Result<Vec<LbPairInformation>> = bin_steps
        .into_iter()
        .map(|bin_step| {
            LB_PAIRS_INFO
                .load(
                    deps.storage,
                    (token_a.unique_key(), token_b.unique_key(), bin_step),
                )
                .map_err(|_| Error::Generic("Error retrieving LbPairInformation".to_string()))
        })
        .collect();

    let response = AllLbPairsResponse {
        lb_pairs_available: lb_pairs_available?,
    };
    to_binary(&response).map_err(Error::CwErr)
}
