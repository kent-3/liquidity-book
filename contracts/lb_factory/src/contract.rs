use crate::{Error, Result};
use cosmwasm_std::{
    entry_point, to_binary, Binary, ContractInfo, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    SubMsgResult, Uint128,
};
use liquidity_book::interfaces::lb_factory::*;
use secret_toolkit::crypto::sha_256;
use std::sync::LazyLock;

mod execute;
mod helper;
mod query;
mod state;

use execute::*;
use query::*;
use state::*;

// TODO: Figure out the role stuff from original.
// I need to learn about Ownable2Step and AccessControl from solidity.

// TODO: see if LazyLock works OK in contracts
// using sha256 instead of keccak256 just because it's more easily available
pub static LB_HOOKS_MANAGER_ROLE: LazyLock<[u8; 32]> =
    LazyLock::new(|| sha_256(b"LB_HOOKS_MANAGER_ROLE"));
static OFFSET_IS_PRESET_OPEN: u8 = 255;
static MIN_BIN_STEP: u8 = 1; // 0.001%
static MAX_FLASH_LOAN_FEE: Uint128 = Uint128::new(10_u128.pow(17)); // 10%
static PUBLIC_VIEWING_KEY: &str = "lb_rocks"; // TODO: decide if this should be public and static

pub const CREATE_LB_PAIR_REPLY_ID: u64 = 1u64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response> {
    let config = State {
        contract_info: ContractInfo {
            address: env.contract.address,
            code_hash: env.contract.code_hash,
        },
        owner: msg.owner.unwrap_or_else(|| info.sender.clone()),
        admin_auth: msg.admin_auth.validate(deps.api)?,
        query_auth: msg.query_auth.validate(deps.api)?,
    };

    STATE.save(deps.storage, &config)?;
    CONTRACT_STATUS.save(deps.storage, &ContractStatus::Active)?;

    FEE_RECIPIENT.save(deps.storage, &msg.fee_recipient)?;
    LB_PAIR_IMPLEMENTATION.save(deps.storage, &Implementation::empty())?;
    LB_TOKEN_IMPLEMENTATION.save(deps.storage, &Implementation::empty())?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    // TODO: recheck this. it doesn't look right. Why would we want to block setting
    // implementations when FreezeAll is active? I think it's meant to be the inverse?
    match CONTRACT_STATUS.load(deps.storage)? {
        ContractStatus::FreezeAll => match msg {
            ExecuteMsg::SetLbPairImplementation { .. }
            | ExecuteMsg::SetLbTokenImplementation { .. } => {
                return Err(Error::TransactionBlock());
            }
            _ => {}
        },
        ContractStatus::Active => {}
    }

    match msg {
        ExecuteMsg::SetLbPairImplementation { implementation } => {
            set_lb_pair_implementation(deps, env, info, implementation)
        }
        // TODO: why isn't this in joe-v2?
        // I think it's because... we don't want the pairs to ever have differing token contract
        // implementations. So really this should probably be static inside the pair contract. I'm
        // not sure there is a way to do that.
        ExecuteMsg::SetLbTokenImplementation { implementation } => {
            set_lb_token_implementation(deps, env, info, implementation)
        }
        ExecuteMsg::CreateLbPair {
            token_x,
            token_y,
            active_id,
            bin_step,
            viewing_key,
            entropy,
        } => create_lb_pair(
            deps,
            env,
            info,
            token_x,
            token_y,
            active_id,
            bin_step,
            viewing_key,
            entropy,
        ),
        ExecuteMsg::SetLbPairIgnored {
            token_x,
            token_y,
            bin_step,
            ignored,
        } => set_lb_pair_ignored(deps, env, info, token_x, token_y, bin_step, ignored),
        ExecuteMsg::SetPreset {
            bin_step,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
            is_open,
        } => set_preset(
            deps,
            env,
            info,
            bin_step,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
            is_open,
        ),
        ExecuteMsg::SetPresetOpenState { bin_step, is_open } => {
            set_preset_open_state(deps, env, info, bin_step, is_open)
        }
        ExecuteMsg::RemovePreset { bin_step } => remove_preset(deps, env, info, bin_step),
        ExecuteMsg::SetFeeParametersOnPair {
            token_x,
            token_y,
            bin_step,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        } => set_fee_parameters_on_pair(
            deps,
            env,
            info,
            token_x,
            token_y,
            bin_step,
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        ),
        ExecuteMsg::SetLBHooksParametersOnPair {
            token_x,
            token_y,
            bin_step,
            hooks_parameters,
            on_hooks_set_data,
        } => set_lb_hooks_parameters_on_pair(
            deps,
            env,
            info,
            token_x,
            token_y,
            bin_step,
            hooks_parameters,
            on_hooks_set_data,
        ),
        ExecuteMsg::RemoveLBHooksOnPair {
            token_x,
            token_y,
            bin_step,
        } => remove_lb_hooks_on_pair(deps, env, info, token_x, token_y, bin_step),
        ExecuteMsg::SetFeeRecipient { fee_recipient } => {
            set_fee_recipient(deps, env, info, fee_recipient)
        }
        ExecuteMsg::SetFlashLoanFee { flash_loan_fee } => {
            set_flash_loan_fee(deps, env, info, flash_loan_fee)
        }
        ExecuteMsg::AddQuoteAsset { asset } => add_quote_asset(deps, env, info, asset),
        ExecuteMsg::RemoveQuoteAsset { asset } => remove_quote_asset(deps, env, info, asset),
        ExecuteMsg::ForceDecay { pair } => force_decay(deps, env, info, pair),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetMinBinStep {} => to_binary(&get_min_bin_step(deps)?),
        QueryMsg::GetFeeRecipient {} => to_binary(&get_fee_recipient(deps)?),
        QueryMsg::GetMaxFlashLoanFee {} => to_binary(&get_max_flash_loan_fee(deps)?),
        QueryMsg::GetFlashLoanFee {} => to_binary(&get_flash_loan_fee(deps)?),
        QueryMsg::GetLbPairImplementation {} => to_binary(&get_lb_pair_implementation(deps)?),
        QueryMsg::GetLbTokenImplementation {} => to_binary(&get_lb_token_implementation(deps)?),
        QueryMsg::GetNumberOfLbPairs {} => to_binary(&get_number_of_lb_pairs(deps)?),
        QueryMsg::GetLbPairAtIndex { index } => to_binary(&get_lb_pair_at_index(deps, index)?),
        QueryMsg::GetNumberOfQuoteAssets {} => to_binary(&get_number_of_quote_assets(deps)?),
        QueryMsg::GetQuoteAssetAtIndex { index } => {
            to_binary(&get_quote_asset_at_index(deps, index)?)
        }
        QueryMsg::IsQuoteAsset { token } => to_binary(&is_quote_asset(deps, token)?),
        QueryMsg::GetLbPairInformation {
            token_x,
            token_y,
            bin_step,
        } => to_binary(&get_lb_pair_information(deps, token_x, token_y, bin_step)?),
        QueryMsg::GetPreset { bin_step } => to_binary(&get_preset(deps, bin_step)?),
        QueryMsg::GetAllBinSteps {} => to_binary(&get_all_bin_steps(deps)?),
        QueryMsg::GetOpenBinSteps {} => to_binary(&get_open_bin_steps(deps)?),
        QueryMsg::GetAllLbPairs { token_x, token_y } => {
            to_binary(&get_all_lb_pairs(deps, token_x, token_y)?)
        }
    }
    .map_err(Error::StdError)
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response> {
    match (msg.id, msg.result) {
        (CREATE_LB_PAIR_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => create_lb_pair_part2(deps, _env, x),
            None => Err(Error::ReplyDataMissing),
        },
        _ => Err(Error::UnknownReplyId { id: msg.id }),
    }
}
