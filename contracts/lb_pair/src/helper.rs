use crate::{prelude::*, state::*};
use cosmwasm_std::{Addr, ContractInfo, CosmosMsg, Deps, Env, QuerierWrapper, StdResult};
use ethnum::U256;
use liquidity_book::{
    interfaces::{lb_router::LiquidityParameters, lb_token},
    libraries::{
        math::{packed_u128_math::PackedUint128Math, u24::U24, uint256_to_u256::ConvertUint256},
        Bytes32,
    },
};
// TODO: sort out viewing key strategy
use secret_toolkit::snip20::{register_receive_msg, set_viewing_key_msg};
use shade_protocol::{
    snip20,
    swap::core::{TokenType, ViewingKey},
};

// TODO: make a 'bin' type with these methods?

pub fn _balance_of(
    querier: QuerierWrapper,
    env: Env,
    token: &ContractInfo,
    viewing_key: String,
) -> u128 {
    querier
        .query_wasm_smart::<secret_toolkit::snip20::Balance>(
            token.code_hash.clone(),
            token.address.clone(),
            &snip20::QueryMsg::Balance {
                address: env.contract.address.to_string(),
                key: viewing_key,
            },
        )
        .map(|response| response.amount.u128())
        .expect("issue querying the contract's snip20 balance")
}

/// Transfers the encoded amounts to the recipient for both tokens.
///
/// # Arguments
///
/// * `amounts` - The amounts, encoded as follows:
///     * [0 - 128[: amount_x
///     * [128 - 256[: amount_y
/// * `token_x` - The token X
/// * `token_y` - The token Y
/// * `recipient` - The recipient
pub fn bin_transfer(
    amounts: Bytes32,
    token_x: TokenType,
    token_y: TokenType,
    recipient: Addr,
) -> Vec<CosmosMsg> {
    let (amount_x, amount_y) = amounts.decode();

    let mut messages: Vec<CosmosMsg> = Vec::with_capacity(2);

    if let Some(msg) = token_x.transfer(amount_x.into(), recipient.clone()) {
        messages.push(msg)
    }

    if let Some(msg) = token_y.transfer(amount_y.into(), recipient) {
        messages.push(msg)
    }

    messages

    // old version that returns an Option

    // let mut messages: Vec<CosmosMsg> = Vec::new();
    //
    // let msgs_x = Self::transfer_x(amounts, token_x, recipient.clone());
    //
    // if let Some(msgs) = msgs_x {
    //     messages.push(msgs);
    // }
    // let msgs_y = Self::transfer_y(amounts, token_y, recipient);
    //
    // if let Some(msgs) = msgs_y {
    //     messages.push(msgs);
    // }
    //
    // if !messages.is_empty() {
    //     Some(messages)
    // } else {
    //     None
    // }
}

/// Transfers the encoded amounts to the recipient, only for token X.
///
/// # Arguments
///
/// * `amounts` - The amounts, encoded as follows:
///     * [0 - 128[: amount_x
///     * [128 - 256[: empty
/// * `token_x` - The token X
/// * `recipient` - The recipient
pub fn bin_transfer_x(amounts: Bytes32, token_x: TokenType, recipient: Addr) -> Option<CosmosMsg> {
    let amount_x = amounts.decode_x();

    token_x.transfer(amount_x.into(), recipient)

    // Raw way to do it, but the TokenType::transfer method takes care of this stuff.
    //
    // if amount > 0 {
    //     match token_x {
    //         TokenType::CustomToken {
    //             contract_addr,
    //             token_code_hash,
    //         } => {
    //             let cosmos_msg = transfer_msg(
    //                 recipient.to_string(),
    //                 amount.into(),
    //                 None,
    //                 None,
    //                 256,
    //                 token_code_hash,
    //                 contract_addr.to_string(),
    //             )
    //             .unwrap();
    //
    //             Some(cosmos_msg)
    //         }
    //
    //         TokenType::NativeToken { denom } => Some(CosmosMsg::Bank(BankMsg::Send {
    //             to_address: recipient.to_string(),
    //             amount: vec![Coin { denom, amount }],
    //         })),
    //     }
    // } else {
    //     None
    // }
}

/// Transfers the encoded amounts to the recipient, only for token Y.
///
/// # Arguments
///
/// * `amounts` - The amounts, encoded as follows:
///     * [0 - 128[: empty
///     * [128 - 256[: amount_y
/// * `token_y` - The token Y
/// * `recipient` - The recipient
pub fn bin_transfer_y(amounts: Bytes32, token_y: TokenType, recipient: Addr) -> Option<CosmosMsg> {
    let amount_y = amounts.decode_y();

    token_y.transfer(amount_y.into(), recipient)
}

pub fn register_pair_token(
    env: &Env,
    messages: &mut Vec<CosmosMsg>,
    token: &TokenType,
    viewing_key: &ViewingKey,
) -> StdResult<()> {
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = token
    {
        messages.push(set_viewing_key_msg(
            viewing_key.to_string(),
            None,
            1,
            token_code_hash.to_string(),
            contract_addr.clone().to_string(),
        )?);
        messages.push(register_receive_msg(
            env.contract.code_hash.clone(),
            None,
            1,
            token_code_hash.to_string(),
            contract_addr.to_string(),
        )?);
    }

    Ok(())
}

pub fn match_lengths(liquidity_parameters: &LiquidityParameters) -> Result<()> {
    if liquidity_parameters.delta_ids.len() != liquidity_parameters.distribution_x.len()
        || liquidity_parameters.delta_ids.len() != liquidity_parameters.distribution_y.len()
    {
        return Err(Error::LengthsMismatch);
    }
    Ok(())
}

pub fn check_ids_bounds(liquidity_parameters: &LiquidityParameters) -> Result<()> {
    if liquidity_parameters.active_id_desired > U24::MAX
        || liquidity_parameters.id_slippage > U24::MAX
    {
        return Err(Error::IdDesiredOverflows {
            id_desired: liquidity_parameters.active_id_desired,
            id_slippage: liquidity_parameters.id_slippage,
        });
    }
    Ok(())
}

pub fn check_active_id_slippage(
    liquidity_parameters: &LiquidityParameters,
    active_id: u32,
) -> Result<()> {
    if liquidity_parameters.active_id_desired + liquidity_parameters.id_slippage < active_id
        || active_id + liquidity_parameters.id_slippage < liquidity_parameters.active_id_desired
    {
        return Err(Error::IdSlippageCaught {
            active_id_desired: liquidity_parameters.active_id_desired,
            id_slippage: liquidity_parameters.id_slippage,
            active_id,
        });
    }
    Ok(())
}

//function won't distinguish between overflow and underflow errors; it'll throw the same DeltaIdOverflows
pub fn calculate_id(
    liquidity_parameters: &LiquidityParameters,
    active_id: u32,
    i: usize,
) -> Result<u32> {
    let id: i64 = active_id as i64 + liquidity_parameters.delta_ids[i];

    if id < 0 || id as u32 > U24::MAX {
        return Err(Error::DeltaIdOverflows {
            delta_id: liquidity_parameters.delta_ids[i],
        });
    }

    Ok(id as u32)
}

pub fn _query_total_supply(deps: Deps, id: u32) -> Result<U256> {
    let lb_token = LB_TOKEN.load(deps.storage)?;

    let msg = lb_token::QueryMsg::IdTotalBalance { id: id.to_string() };

    let res = deps.querier.query_wasm_smart::<lb_token::QueryAnswer>(
        lb_token.code_hash,
        lb_token.address.to_string(),
        &msg,
    )?;

    let total_supply_uint256 = match res {
        lb_token::QueryAnswer::IdTotalBalance { amount } => amount,
        _ => return Err(Error::Generic("Wrong response for lb_token".to_string())),
    };

    Ok(total_supply_uint256.uint256_to_u256())
}

pub fn query_token_symbol(deps: Deps, code_hash: String, address: Addr) -> Result<String> {
    let msg = snip20::QueryMsg::TokenInfo {};

    let res = deps.querier.query_wasm_smart::<snip20::QueryAnswer>(
        code_hash,
        address.to_string(),
        &(&msg),
    )?;

    let symbol = match res {
        snip20::QueryAnswer::TokenInfo { symbol, .. } => symbol,
        _ => return Err(Error::Generic(format!("Token {} not valid", address))),
    };

    Ok(symbol)
}

/// Returns id of the next non-empty bin.
///
/// # Arguments
/// * `swap_for_y Whether the swap is for Y
/// * `id` - The id of the bin
pub fn _get_next_non_empty_bin(deps: Deps, swap_for_y: bool, id: u32) -> u32 {
    if swap_for_y {
        TREE.find_first_right(deps.storage, id)
    } else {
        TREE.find_first_left(deps.storage, id)
    }
}

pub fn only_factory(sender: &Addr, factory: &Addr) -> Result<()> {
    if sender != factory {
        return Err(Error::OnlyFactory);
    }

    Ok(())
}

pub fn only_protocol_fee_recipient(sender: &Addr, recipient: &Addr) -> Result<()> {
    if sender != recipient {
        return Err(Error::OnlyProtocolFeeRecipient);
    }

    Ok(())
}
