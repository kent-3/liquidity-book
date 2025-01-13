use super::{
    helper::{_get_lb_pair_information, _is_preset_open, _sort_tokens},
    state::*,
    MAX_FLASH_LOAN_FEE, MIN_BIN_STEP, OFFSET_IS_PRESET_OPEN,
};
use crate::{Error, Result};
use cosmwasm_std::{Deps, StdResult};
use liquidity_book::{
    core::TokenType, interfaces::lb_factory::*, libraries::math::encoded::Encoded,
};

/// Get the minimum bin step a pair can have.
pub fn get_min_bin_step(_deps: Deps) -> Result<MinBinStepResponse> {
    Ok(MinBinStepResponse {
        min_bin_step: MIN_BIN_STEP,
    })
}

/// Get the protocol fee recipient.
pub fn get_fee_recipient(deps: Deps) -> Result<FeeRecipientResponse> {
    Ok(FeeRecipientResponse {
        fee_recipient: FEE_RECIPIENT.load(deps.storage)?,
    })
}

/// Get the maximum fee percentage for flashLoans.
pub fn get_max_flash_loan_fee(_deps: Deps) -> Result<MaxFlashLoanFeeResponse> {
    Ok(MaxFlashLoanFeeResponse {
        max_flash_loan_fee: MAX_FLASH_LOAN_FEE,
    })
}

/// Get the fee for flash loans, in 1e18.
pub fn get_flash_loan_fee(deps: Deps) -> Result<FlashLoanFeeResponse> {
    Ok(FlashLoanFeeResponse {
        flash_loan_fee: FLASH_LOAN_FEE.load(deps.storage)?,
    })
}

/// Get the code ID and hash of the LbPair implementation.
pub fn get_lb_pair_implementation(deps: Deps) -> Result<LbPairImplementationResponse> {
    Ok(LbPairImplementationResponse {
        lb_pair_implementation: LB_PAIR_IMPLEMENTATION.load(deps.storage)?,
    })
}

/// Get the code ID and hash of the LbToken implementation.
pub fn get_lb_token_implementation(deps: Deps) -> Result<LbTokenImplementationResponse> {
    Ok(LbTokenImplementationResponse {
        lb_token_implementation: LB_TOKEN_IMPLEMENTATION.load(deps.storage)?,
    })
}

/// Returns the number of LbPairs created.
pub fn get_number_of_lb_pairs(deps: Deps) -> Result<NumberOfLbPairsResponse> {
    Ok(NumberOfLbPairsResponse {
        lb_pair_number: ALL_LB_PAIRS.get_len(deps.storage)?,
    })
}

/// Returns the LbPair created at the given index.
pub fn get_lb_pair_at_index(deps: Deps, index: u32) -> Result<LbPairAtIndexResponse> {
    Ok(LbPairAtIndexResponse {
        lb_pair: ALL_LB_PAIRS.get_at(deps.storage, index)?,
    })
}

/// Returns the number of quote assets whitelisted.
pub fn get_number_of_quote_assets(deps: Deps) -> Result<NumberOfQuoteAssetsResponse> {
    Ok(NumberOfQuoteAssetsResponse {
        number_of_quote_assets: QUOTE_ASSET_WHITELIST.get_len(deps.storage)?,
    })
}

/// Returns the quote asset whitelisted at the given index.
pub fn get_quote_asset_at_index(deps: Deps, index: u32) -> Result<QuoteAssetAtIndexResponse> {
    Ok(QuoteAssetAtIndexResponse {
        asset: QUOTE_ASSET_WHITELIST.get_at(deps.storage, index)?,
    })
}

/// Returns whether a token is a quote asset (true) or not (false).
pub fn is_quote_asset(deps: Deps, token: TokenType) -> Result<IsQuoteAssetResponse> {
    let is_quote = QUOTE_ASSET_WHITELIST
        .iter(deps.storage)?
        .any(|result| match result {
            Ok(t) => t.eq(&token),
            Err(_) => false,
        });

    Ok(IsQuoteAssetResponse { is_quote })
}

// TODO: should this return None instead of default?
/// Returns the LbPairInformation if it exists, if not, then empty information is returned. The order doesn't matter.
pub fn get_lb_pair_information(
    deps: Deps,
    token_a: TokenType,
    token_b: TokenType,
    bin_step: u16,
) -> Result<LbPairInformationResponse> {
    let lb_pair_information =
        _get_lb_pair_information(deps, &token_a, &token_b, bin_step).unwrap_or_default();

    Ok(LbPairInformationResponse {
        lb_pair_information,
    })
}

/// Returns the different parameters of the preset.
pub fn get_preset(deps: Deps, bin_step: u16) -> Result<PresetResponse> {
    if !PRESETS.contains(deps.storage, &bin_step) {
        return Err(Error::BinStepHasNoPreset { bin_step });
    }

    // NOTE: each preset is an encoded Bytes32.
    // The PairParameters wrapper provides methods to decode specific values.
    let preset = PRESETS.get(deps.storage, &bin_step).unwrap();

    let base_factor = preset.get_base_factor();
    let filter_period = preset.get_filter_period();
    let decay_period = preset.get_decay_period();
    let reduction_factor = preset.get_reduction_factor();
    let variable_fee_control = preset.get_variable_fee_control();
    let protocol_share = preset.get_protocol_share();
    let max_volatility_accumulator = preset.get_max_volatility_accumulator();

    let is_open = preset.0.decode_bool(OFFSET_IS_PRESET_OPEN);

    Ok(PresetResponse {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
        is_open,
    })
}

/// Returns the list of available bin steps with a preset.
pub fn get_all_bin_steps(deps: Deps) -> Result<AllBinStepsResponse> {
    let bin_step_with_preset: Vec<u16> = PRESET_BIN_STEPS
        .iter(deps.storage)?
        .filter_map(|result| result.ok())
        .collect();

    Ok(AllBinStepsResponse {
        bin_step_with_preset,
    })
}

// this does the same thing as `get_all_bin_steps` but returns only the ones where `is_open` is true
/// Returns the list of open bin steps.
pub fn get_open_bin_steps(deps: Deps) -> Result<OpenBinStepsResponse> {
    // TODO: revisit this once we have an EnumerableMap type of storage
    // let hashset = PRESET_HASHSET.load(deps.storage)?;

    let bin_steps: Vec<u16> = PRESET_BIN_STEPS
        .iter(deps.storage)?
        .collect::<StdResult<Vec<u16>>>()?;

    // more concise but difficult to read
    //
    // let open_bin_steps: Vec<u16> = PRESET_BIN_STEPS
    //     .iter(deps.storage)?
    //     .filter_map(|result| {
    //         // Handle the outer error from the iterator
    //         result.ok().and_then(|bin_step| {
    //             // Get the preset and handle potential error with ok()
    //             PRESETS
    //                 .get(deps.storage, &bin_step)
    //                 .filter(|preset| _is_preset_open(preset.0)) // Keep only open presets
    //                 .map(|_| bin_step) // Return the bin_step if preset is open
    //         })
    //     })
    //     .collect();

    let mut open_bin_steps = Vec::<u16>::new();

    for bin_step in bin_steps {
        let preset = PRESETS.get(deps.storage, &bin_step).unwrap_or_default();

        if _is_preset_open(preset.0) {
            open_bin_steps.push(bin_step)
        }
    }

    Ok(OpenBinStepsResponse { open_bin_steps })
}

/// Returns all the LbPair of a pair of tokens.
pub fn get_all_lb_pairs(
    deps: Deps,
    token_x: TokenType,
    token_y: TokenType,
) -> Result<AllLbPairsResponse> {
    let (token_a, token_b) = _sort_tokens(token_x, token_y);

    let bin_steps = AVAILABLE_LB_PAIR_BIN_STEPS
        .get(deps.storage, &(token_a.unique_key(), token_b.unique_key()))
        .unwrap_or_default();

    let lb_pairs_available: Vec<LbPairInformation> = bin_steps
        .into_iter()
        .filter_map(|bin_step| {
            LB_PAIRS_INFO.get(
                deps.storage,
                &(token_a.unique_key(), token_b.unique_key(), bin_step),
            )
        })
        .collect::<Vec<LbPairInformation>>();

    // NOTE: This cannot fail, but I'm keeping it `Result` to match all the other queries.
    Ok(AllLbPairsResponse { lb_pairs_available })
}
