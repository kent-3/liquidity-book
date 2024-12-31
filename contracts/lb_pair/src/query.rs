use crate::{helper::*, state::*, Error, Result};
use cosmwasm_std::{Addr, Deps, Env, Uint128, Uint256};
use ethnum::U256;
use liquidity_book::{
    interfaces::lb_pair::*,
    libraries::{
        constants::SCALE_OFFSET,
        math::{
            u24::U24,
            uint256_to_u256::{ConvertU256, ConvertUint256},
        },
        oracle_helper, BinHelper, Bytes32, FeeHelper, OracleMap, PackedUint128Math, PriceHelper,
        U256x256Math,
    },
};
// TODO: get rid of these dependencies
use shade_protocol::{
    swap::{
        amm_pair::{
            FeeInfo,
            QueryMsgResponse::{self, GetPairInfo},
        },
        core::{Fee, TokenPair},
    },
    Contract,
};

// TODO: Revisit if this function is necessary. It seems like something that might belong in the
//       lb-factory contract. It should at least have it's own interface and not use amm_pair's.
pub fn query_pair_info(deps: Deps) -> Result<QueryMsgResponse> {
    let factory = FACTORY.load(deps.storage)?;
    let lb_token = LB_TOKEN.load(deps.storage)?;
    let token_x = TOKEN_X.load(deps.storage)?;
    let token_y = TOKEN_Y.load(deps.storage)?;

    let bin_step = BIN_STEP.load(deps.storage)?;

    let (reserve_x, reserve_y) = RESERVES.load(deps.storage)?.decode();
    let parameters = PARAMETERS.load(deps.storage)?;

    let response = GetPairInfo {
        liquidity_token: Contract {
            address: lb_token.address,
            code_hash: lb_token.code_hash,
        },
        factory: Some(Contract {
            address: factory.address.clone(),
            code_hash: factory.code_hash.clone(),
        }),
        pair: TokenPair(token_x, token_y, false),
        amount_0: Uint128::from(reserve_x),
        amount_1: Uint128::from(reserve_y),
        total_liquidity: Uint128::default(), // no global liquidity, liquidity is calculated on per bin basis
        contract_version: 1, // TODO set this like const AMM_PAIR_CONTRACT_VERSION: u32 = 1;
        fee_info: FeeInfo {
            shade_dao_address: Addr::unchecked(""), // TODO set shade dao address
            lp_fee: Fee {
                // TODO set this
                nom: parameters.get_base_fee(bin_step) as u64,
                denom: 1_000_000_000_000_000_000,
            },
            shade_dao_fee: Fee {
                nom: parameters.get_base_fee(bin_step) as u64,
                denom: 1_000_000_000_000_000_000,
            },
            stable_lp_fee: Fee {
                nom: parameters.get_base_fee(bin_step) as u64,
                denom: 1_000_000_000_000_000_000,
            },
            stable_shade_dao_fee: Fee {
                nom: parameters.get_base_fee(bin_step) as u64,
                denom: 1_000_000_000_000_000_000,
            },
        },
        stable_info: None,
    };

    Ok(response)
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
pub fn query_factory(deps: Deps) -> Result<FactoryResponse> {
    let factory = FACTORY.load(deps.storage)?;

    let response = FactoryResponse {
        factory: factory.address.clone(),
    };

    Ok(response)
}

// TODO: this returns a ContractInfo, but the factory one only returns an Addr
pub fn query_lb_token(deps: Deps) -> Result<LbTokenResponse> {
    let lb_token = LB_TOKEN.load(deps.storage)?;

    let response = LbTokenResponse { lb_token };

    Ok(response)
}

/// Returns the token X of the Liquidity Book Pair.
///
/// # Returns
///
/// * `token_x` - The address of the token X
pub fn query_token_x(deps: Deps) -> Result<TokenXResponse> {
    let token_x = TOKEN_X.load(deps.storage)?;

    let response = TokenXResponse { token_x };

    Ok(response)
}

/// Returns the token Y of the Liquidity Book Pair.
///
/// # Returns
///
/// * `token_y` - The address of the token Y
pub fn query_token_y(deps: Deps) -> Result<TokenYResponse> {
    let token_y = TOKEN_Y.load(deps.storage)?;

    let response = TokenYResponse { token_y };

    Ok(response)
}

/// Returns the bin_step of the Liquidity Book Pair.
///
/// The bin step is the increase in price between two consecutive bins, in basis points.
/// For example, a bin step of 1 means that the price of the next bin is 0.01% higher than the price of the previous bin.
///
/// # Returns
///
/// * `bin_step` - The bin step of the Liquidity Book Pair, in 10_000th
pub fn query_bin_step(deps: Deps) -> Result<BinStepResponse> {
    let bin_step = BIN_STEP.load(deps.storage)?;

    let response = BinStepResponse { bin_step };

    Ok(response)
}

/// Returns the reserves of the Liquidity Book Pair.
///
/// This is the sum of the reserves of all bins, minus the protocol fees.
///
/// # Returns
///
/// * `reserve_x` - The reserve of token X
/// * `reserve_y` - The reserve of token Y
pub fn query_reserves(deps: Deps) -> Result<ReservesResponse> {
    let reserves = RESERVES.load(deps.storage)?;
    let protocol_fees = PROTOCOL_FEES.load(deps.storage)?;
    let (reserve_x, reserve_y) = reserves.sub(protocol_fees)?.decode();

    let response = ReservesResponse {
        reserve_x: reserve_x.into(),
        reserve_y: reserve_y.into(),
    };

    Ok(response)
}

/// Returns the active id of the Liquidity Book Pair.
///
/// The active id is the id of the bin that is currently being used for swaps.
/// The price of the active bin is the price of the Liquidity Book Pair and can be calculated as follows:
/// `price = (1 + binStep / 10_000) ^ (activeId - 2^23)`
///
/// # Returns
///
/// * `active_id` - The active id of the Liquidity Book Pair
pub fn query_active_id(deps: Deps) -> Result<ActiveIdResponse> {
    let active_id = PARAMETERS.load(deps.storage)?.get_active_id();

    let response = ActiveIdResponse { active_id };

    Ok(response)
}

/// Returns the reserves of a bin.
///
/// # Arguments
///
/// * `id` - The id of the bin
///
/// # Returns
///
/// * `bin_reserve_x` - The reserve of token X in the bin
/// * `bin_reserve_y` - The reserve of token Y in the bin
pub fn query_bin(deps: Deps, id: u32) -> Result<BinResponse> {
    let bin_reserves = BINS.get(deps.storage, &id).unwrap_or([0u8; 32]);
    let (bin_reserve_x, bin_reserve_y) = bin_reserves.decode();

    let response = BinResponse {
        bin_reserve_x: bin_reserve_x.into(),
        bin_reserve_y: bin_reserve_y.into(),
        bin_id: id,
    };

    Ok(response)
}

/// Returns the reserves of many bins.
///
/// # Arguments
///
/// * `ids` - A list of bin ids
///
/// # Returns
///
/// * `bin_reserve_x` - The reserve of token X in the bin
/// * `bin_reserve_y` - The reserve of token Y in the bin
pub fn query_bins(deps: Deps, ids: Vec<u32>) -> Result<BinsResponse> {
    let mut bin_responses = Vec::new();
    for id in ids {
        let bin: Bytes32 = BINS.get(deps.storage, &id).unwrap_or([0u8; 32]);
        let (bin_reserve_x, bin_reserve_y) = bin.decode();
        bin_responses.push(BinResponse {
            bin_reserve_x: bin_reserve_x.into(),
            bin_reserve_y: bin_reserve_y.into(),
            bin_id: id,
        });
    }

    let response: BinsResponse = BinsResponse(bin_responses);

    Ok(response)
}

pub fn query_all_bins(
    deps: Deps,
    env: Env,
    page: Option<u32>,
    page_size: Option<u32>,
    id: Option<u32>,
) -> Result<AllBinsResponse> {
    let page = page.unwrap_or(0);
    let page_size = page_size.unwrap_or(10);

    let mut id = id.unwrap_or(0u32);
    let mut bin_responses = Vec::new();

    // let tree = BIN_TREE.load(deps.storage)?;
    let total = if page > 0 {
        page * page_size
    } else {
        page_size
    };

    let mut counter: u32 = 0;

    loop {
        let next_id = TREE.find_first_left(deps.storage, id);
        id = next_id;

        if next_id == 0 || next_id == U24::MAX {
            break;
        }

        let (bin_reserve_x, bin_reserve_y) =
            BINS.get(deps.storage, &id).unwrap_or_default().decode();
        bin_responses.push(BinResponse {
            bin_reserve_x: bin_reserve_x.into(),
            bin_reserve_y: bin_reserve_y.into(),
            bin_id: id,
        });
        counter += 1;

        if counter == total {
            break;
        }
    }
    let response = AllBinsResponse {
        reserves: bin_responses,
        last_id: id,
        current_block_height: env.block.height,
    };

    Ok(response)
}

/// Returns the next non-empty bin.
///
/// The next non-empty bin is the bin with a higher (if swap_for_y is true) or lower (if swap_for_y is false)
/// id that has a non-zero reserve of token X or Y.
///
/// # Arguments
///
/// * `swap_for_y` - Whether the swap is for token Y (true) or token X (false
/// * `id` - The id of the bin
///
/// # Returns
///
/// * `next_id` - The id of the next non-empty bin
pub fn query_next_non_empty_bin(
    deps: Deps,
    swap_for_y: bool,
    id: u32,
) -> Result<NextNonEmptyBinResponse> {
    // let tree = BIN_TREE.load(deps.storage)?;
    let next_id = _get_next_non_empty_bin(deps, swap_for_y, id);

    let response = NextNonEmptyBinResponse { next_id };

    Ok(response)
}

/// Returns the protocol fees of the Liquidity Book Pair.
///
/// # Returns
///
/// * `protocol_fee_x` - The protocol fees of token X
/// * `protocol_fee_y` - The protocol fees of token Y
pub fn query_protocol_fees(deps: Deps) -> Result<ProtocolFeesResponse> {
    let (protocol_fee_x, protocol_fee_y) = PROTOCOL_FEES.load(deps.storage)?.decode();

    let response = ProtocolFeesResponse {
        protocol_fee_x,
        protocol_fee_y,
    };

    Ok(response)
}

/// Returns the static fee parameters of the Liquidity Book Pair.
///
/// # Returns
///
/// * `base_factor` - The base factor for the static fee
/// * `filter_period` - The filter period for the static fee
/// * `decay_period` - The decay period for the static fee
/// * `reduction_factor` - The reduction factor for the static fee
/// * `variable_fee_control` - The variable fee control for the static fee
/// * `protocol_share` - The protocol share for the static fee
/// * `max_volatility_accumulator` - The maximum volatility accumulator for the static fee
pub fn query_static_fee_parameters(deps: Deps) -> Result<StaticFeeParametersResponse> {
    let parameters = PARAMETERS.load(deps.storage)?;

    let response = StaticFeeParametersResponse {
        base_factor: parameters.get_base_factor(),
        filter_period: parameters.get_filter_period(),
        decay_period: parameters.get_decay_period(),
        reduction_factor: parameters.get_reduction_factor(),
        variable_fee_control: parameters.get_variable_fee_control(),
        protocol_share: parameters.get_protocol_share(),
        max_volatility_accumulator: parameters.get_max_volatility_accumulator(),
    };

    Ok(response)
}

/// Returns the variable fee parameters of the Liquidity Book Pair.
///
/// # Returns
///
/// * `volatility_accumulator` - The volatility accumulator for the variable fee
/// * `volatility_reference` - The volatility reference for the variable fee
/// * `id_reference` - The id reference for the variable fee
/// * `time_of_last_update` - The time of last update for the variable fee
pub fn query_variable_fee_parameters(deps: Deps) -> Result<VariableFeeParametersResponse> {
    let parameters = PARAMETERS.load(deps.storage)?;

    let response = VariableFeeParametersResponse {
        volatility_accumulator: parameters.get_volatility_accumulator(),
        volatility_reference: parameters.get_volatility_reference(),
        id_reference: parameters.get_id_reference(),
        time_of_last_update: parameters.get_time_of_last_update(),
    };

    Ok(response)
}

/// Returns the oracle parameters of the Liquidity Book Pair.
///
/// # Returns
///
/// * `sample_lifetime` - The sample lifetime for the oracle
/// * `size` - The size of the oracle
/// * `active_size` - The active size of the oracle
/// * `last_updated` - The last updated timestamp of the oracle
/// * `first_timestamp` - The first timestamp of the oracle, i.e. the timestamp of the oldest sample
pub fn query_oracle_params(deps: Deps) -> Result<OracleParametersResponse> {
    let parameters = PARAMETERS.load(deps.storage)?;

    let sample_lifetime = oracle_helper::MAX_SAMPLE_LIFETIME;

    let oracle_id = parameters.get_oracle_id();

    if oracle_id > 0 {
        let (mut sample, mut active_size) =
            ORACLE.get_active_sample_and_size(deps.storage, oracle_id)?;

        let size = sample.get_oracle_length();
        let last_updated = sample.get_sample_last_update();

        if last_updated == 0 {
            active_size = 0;
        }

        let mut first_timestamp = 0u64;

        // TODO: check if the +1 is correct here
        if active_size > 0 {
            sample = ORACLE.get_sample(deps.storage, 1 + (oracle_id % active_size))?;
            first_timestamp = sample.get_sample_last_update();
        }

        Ok(OracleParametersResponse {
            sample_lifetime,
            size,
            active_size,
            last_updated,
            first_timestamp,
        })
    } else {
        Ok(OracleParametersResponse {
            sample_lifetime,
            ..Default::default()
        })
    }
}

/// Returns the cumulative values of the Liquidity Book Pair at a given timestamp.
///
/// # Arguments
///
/// * `lookup_timestamp` - The timestamp at which to look up the cumulative values
//
/// # Returns
///
/// * `cumulative_id` - The cumulative id of the Liquidity Book Pair at the given timestamp
/// * `cumulative_volatility` - The cumulative volatility of the Liquidity Book Pair at the given timestamp
/// * `cumulative_bin_crossed` - The cumulative bin crossed of the Liquidity Book Pair at the given timestamp
pub fn query_oracle_sample_at(
    deps: Deps,
    env: Env,
    lookup_timestamp: u64,
) -> Result<OracleSampleAtResponse> {
    let mut parameters = PARAMETERS.load(deps.storage)?;

    let oracle_id = parameters.get_oracle_id();

    if oracle_id == 0 || lookup_timestamp > env.block.time.seconds() {
        return Ok(OracleSampleAtResponse {
            sample: OracleSampleResponse {
                cumulative_id: 0,
                cumulative_volatility: 0,
                cumulative_bin_crossed: 0,
            },
        });
    }

    let (time_of_last_update, mut cumulative_id, mut cumulative_volatility, cumulative_bin_crossed) =
        ORACLE.get_sample_at(deps.storage, oracle_id, lookup_timestamp)?;

    if time_of_last_update < lookup_timestamp {
        parameters.update_volatility_parameters(parameters.get_active_id(), lookup_timestamp)?;

        let delta_time = lookup_timestamp - time_of_last_update;

        cumulative_id += parameters.get_active_id() as u64 * delta_time;
        cumulative_volatility += parameters.get_volatility_accumulator() as u64 * delta_time;
    }

    Ok(OracleSampleAtResponse {
        sample: OracleSampleResponse {
            cumulative_id,
            cumulative_volatility,
            cumulative_bin_crossed,
        },
    })
}

/// Returns the price corresponding to the given id, as a 128.128-binary fixed-point number.
///
/// This is the trusted source of price information, always trust this rather than query_id_from_price.
///
/// # Arguments
///
/// * `id` - The id of the bin
///
/// # Returns
///
/// * `price` - The price corresponding to this id
pub fn query_price_from_id(deps: Deps, id: u32) -> Result<PriceFromIdResponse> {
    let bin_step = BIN_STEP.load(deps.storage)?;
    let price = PriceHelper::get_price_from_id(id, bin_step)?.u256_to_uint256();

    let response = PriceFromIdResponse { price };

    Ok(response)
}

/// Returns the id corresponding to the given price.
///
/// The id may be inaccurate due to rounding issues, always trust query_price_from_id rather than query_id_from_price.
///
/// # Arguments
///
/// * `price` - The price of y per x as a 128.128-binary fixed-point number
///
/// # Returns
///
/// * `id` - The id of the bin corresponding to this price
pub fn query_id_from_price(deps: Deps, price: Uint256) -> Result<IdFromPriceResponse> {
    let bin_step = BIN_STEP.load(deps.storage)?;
    let id = PriceHelper::get_id_from_price(price.uint256_to_u256(), bin_step)?;

    let response = IdFromPriceResponse { id };

    Ok(response)
}

/// Simulates a swap in.
///
/// # Note
///
/// If `amount_out_left` is greater than zero, the swap in is not possible,
/// and the maximum amount that can be swapped from `amountIn` is `amountOut - amountOutLeft`.
///
/// # Arguments
///
/// * `amount_out` - The amount of token X or Y to swap in
/// * `swap_for_y` - Whether the swap is for token Y (true) or token X (false)
///
/// # Returns
/// * `amount_in` - The amount of token X or Y that can be swapped in, including the fee
/// * `amount_out_left` - The amount of token Y or X that cannot be swapped out
/// * `fee` - The fee of the swap
pub fn query_swap_in(
    deps: Deps,
    env: Env,
    amount_out: u128,
    swap_for_y: bool,
) -> Result<SwapInResponse> {
    let mut amount_in = 0u128;
    let mut amount_out_left = amount_out;
    let mut fee = 0u128;

    let bin_step = BIN_STEP.load(deps.storage)?;
    // let tree = BIN_TREE.load(deps.storage)?;

    let mut parameters = PARAMETERS.load(deps.storage)?;

    let mut id = parameters.get_active_id();

    parameters.update_references(env.block.time.seconds())?;

    loop {
        let bin_reserves = BINS
            .get(deps.storage, &id)
            .unwrap_or_default()
            .decode_alt(!swap_for_y);

        if bin_reserves > 0 {
            let price = PriceHelper::get_price_from_id(id, bin_step)?;

            let amount_out_of_bin = if bin_reserves > amount_out_left {
                amount_out_left
            } else {
                bin_reserves
            };

            parameters.update_volatility_accumulator(id)?;

            let amount_in_without_fee = if swap_for_y {
                U256::from(amount_out_of_bin).shift_div_round_up(SCALE_OFFSET, price)?
            } else {
                U256::from(amount_out_of_bin).mul_shift_round_up(price, SCALE_OFFSET)?
            }
            .as_u128();

            let total_fee = parameters.get_total_fee(bin_step)?;
            let fee_amount = amount_in_without_fee.get_fee_amount(total_fee)?;

            amount_in += amount_in_without_fee + fee_amount;
            amount_out_left -= amount_out_of_bin;

            fee += fee_amount;
        }

        if amount_out_left == 0 {
            break;
        } else {
            let next_id = _get_next_non_empty_bin(deps, swap_for_y, id);

            if next_id == 0 || next_id == U24::MAX {
                break;
            }

            id = next_id;
        }
    }

    let response = SwapInResponse {
        amount_in: Uint128::from(amount_in),
        amount_out_left: Uint128::from(amount_out_left),
        fee: Uint128::from(fee),
    };

    Ok(response)
}

/// Simulates a swap out.
///
/// # Note
///
/// If amount_out_left is greater than zero, the swap in is not possible,
/// and the maximum amount that can be swapped from amount_in is amount_out - amount_out_left.
///
/// # Arguments
///
/// * `amount_in` - The amount of token X or Y to swap in
/// * `swap_for_y` - Whether the swap is for token Y (true) or token X (false)
///
/// # Returns
/// * `amount_in_left` - The amount of token X or Y that cannot be swapped in
/// * `amount_out` - The amount of token Y or X that can be swapped out
/// * `fee` - The fee of the swap
pub fn query_swap_out(
    deps: Deps,
    env: Env,
    amount_in: u128,
    swap_for_y: bool,
) -> Result<SwapOutResponse> {
    let mut amounts_in_left = Bytes32::encode_alt(amount_in, swap_for_y);
    let mut amounts_out = 0u128;
    let mut fee = 0u128;

    let bin_step = BIN_STEP.load(deps.storage)?;
    // let tree = BIN_TREE.load(deps.storage)?;

    let mut parameters = PARAMETERS.load(deps.storage)?;

    let mut id = parameters.get_active_id();

    parameters.update_references(env.block.time.seconds())?;

    loop {
        let bin_reserves = BINS.get(deps.storage, &id).unwrap_or_default();
        if !bin_reserves.is_empty(!swap_for_y) {
            parameters.update_volatility_accumulator(id)?;

            let (amounts_in_with_fees, amounts_out_of_bin, total_fees) =
                bin_reserves.get_amounts(parameters, bin_step, swap_for_y, id, amounts_in_left)?;

            if amounts_in_with_fees > [0u8; 32] {
                amounts_in_left = amounts_in_left.sub(amounts_in_with_fees)?;
                amounts_out += amounts_out_of_bin.decode_alt(!swap_for_y);
                fee += total_fees.decode_alt(swap_for_y);
            }
        }

        if amounts_in_left == [0u8; 32] {
            break;
        } else {
            let next_id = _get_next_non_empty_bin(deps, swap_for_y, id);

            if next_id == 0 || next_id == U24::MAX {
                break;
            }

            id = next_id;
        }
    }

    let amount_in_left = Bytes32::decode_alt(&amounts_in_left, swap_for_y);

    let response = SwapOutResponse {
        amount_in_left: Uint128::from(amount_in_left),
        amount_out: Uint128::from(amounts_out),
        fee: Uint128::from(fee),
    };

    Ok(response)
}

/// Returns the Liquidity Book Factory.
///
/// # Returns
///
/// * `factory` - The Liquidity Book Factory
pub fn query_total_supply(deps: Deps, id: u32) -> Result<LbTokenSupplyResponse> {
    let total_supply = _query_total_supply(deps, id)?.u256_to_uint256();

    let response = LbTokenSupplyResponse { total_supply };

    Ok(response)
}
