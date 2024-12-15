use crate::prelude::*;
use cosmwasm_std::{
    entry_point, to_binary, Binary, ContractInfo, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsgResult,
};
use lb_interfaces::{
    lb_factory::*,
    lb_pair::{LbPair, LbPairInformation},
};
use std::collections::HashSet;

mod execute;
mod helper;
mod query;
mod state;

use execute::*;
use query::*;
use state::*;

static OFFSET_IS_PRESET_OPEN: u8 = 255;
static MIN_BIN_STEP: u8 = 1; // 0.001%
                             // FIXME: thats a bitxor not pow
static MAX_FLASHLOAN_FEE: u8 = 10 ^ 17; // 10%

const INSTANTIATE_REPLY_ID: u64 = 1u64;

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
        fee_recipient: msg.fee_recipient,
        lb_pair_implementation: ContractImplementation::default(),
        lb_token_implementation: ContractImplementation::default(),
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        query_auth: msg.query_auth.into_valid(deps.api)?,
    };

    STATE.save(deps.storage, &config)?;
    // TODO: is this necessary?
    PRESET_HASHSET.save(deps.storage, &HashSet::new())?;
    CONTRACT_STATUS.save(deps.storage, &ContractStatus::Active)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<Response> {
    // TODO: recheck this. it doesn't look right.
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
        } => set_pair_preset(
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
        ExecuteMsg::SetLBHooksParametersOnPair => todo!(),
        ExecuteMsg::RemoveLBHooksOnPair => todo!(),
        ExecuteMsg::SetFeeRecipient { fee_recipient } => {
            set_fee_recipient(deps, env, info, fee_recipient)
        }
        ExecuteMsg::SetFlashLoanFee => todo!(),
        ExecuteMsg::AddQuoteAsset { asset } => add_quote_asset(deps, env, info, asset),
        ExecuteMsg::RemoveQuoteAsset { asset } => remove_quote_asset(deps, env, info, asset),
        ExecuteMsg::ForceDecay { pair } => force_decay(deps, env, info, pair),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetMinBinStep {} => to_binary(&query_min_bin_step(deps)?),
        QueryMsg::GetFeeRecipient {} => to_binary(&query_fee_recipient(deps)?),
        QueryMsg::GetMaxFlashLoanFee {} => to_binary(&query_max_flash_loan_fee(deps)?),
        QueryMsg::GetFlashLoanFee {} => to_binary(&query_flash_loan_fee(deps)?),
        QueryMsg::GetLbPairImplementation {} => to_binary(&query_lb_pair_implementation(deps)?),
        QueryMsg::GetLbTokenImplementation {} => to_binary(&query_lb_token_implementation(deps)?),
        QueryMsg::GetNumberOfLbPairs {} => to_binary(&query_number_of_lb_pairs(deps)?),
        QueryMsg::GetLbPairAtIndex { index } => to_binary(&query_lb_pair_at_index(deps, index)?),
        QueryMsg::GetNumberOfQuoteAssets {} => to_binary(&query_number_of_quote_assets(deps)?),
        QueryMsg::GetQuoteAssetAtIndex { index } => {
            to_binary(&query_quote_asset_at_index(deps, index)?)
        }
        QueryMsg::IsQuoteAsset { token } => to_binary(&query_is_quote_asset(deps, token)?),
        QueryMsg::GetLbPairInformation {
            token_x,
            token_y,
            bin_step,
        } => to_binary(&query_lb_pair_information(
            deps, token_x, token_y, bin_step,
        )?),
        QueryMsg::GetPreset { bin_step } => to_binary(&query_preset(deps, bin_step)?),
        QueryMsg::GetAllBinSteps {} => to_binary(&query_all_bin_steps(deps)?),
        QueryMsg::GetOpenBinSteps {} => to_binary(&query_open_bin_steps(deps)?),
        QueryMsg::GetAllLbPairs { token_x, token_y } => {
            to_binary(&query_all_lb_pairs(deps, token_x, token_y)?)
        }
    }
    .map_err(Error::CwErr)
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match (msg.id, msg.result) {
        (INSTANTIATE_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let contract_address = deps.api.addr_validate(&String::from_utf8(x.to_vec())?)?;
                let lb_pair_key = ephemeral_storage_r(deps.storage).load()?;

                let token_a = lb_pair_key.token_a;
                let token_b = lb_pair_key.token_b;
                let bin_step = lb_pair_key.bin_step;
                let code_hash = lb_pair_key.code_hash;

                let lb_pair = LbPair {
                    token_x: token_a.clone(),
                    token_y: token_b.clone(),
                    bin_step,
                    contract: ContractInfo {
                        address: contract_address,
                        code_hash,
                    },
                };

                LB_PAIRS_INFO.save(
                    deps.storage,
                    (token_a.unique_key(), token_b.unique_key(), bin_step),
                    &LbPairInformation {
                        bin_step: lb_pair_key.bin_step,
                        lb_pair: lb_pair.clone(),
                        created_by_owner: lb_pair_key.is_open,
                        ignored_for_routing: false,
                    },
                )?;

                ALL_LB_PAIRS.push(deps.storage, &lb_pair)?;

                // load the different bin_step LbPairs that exist for this pair of tokens, then add the new one
                let mut bin_step_list = AVAILABLE_LB_PAIR_BIN_STEPS
                    .load(deps.storage, (token_a.unique_key(), token_b.unique_key()))
                    .unwrap_or_default();
                bin_step_list.insert(bin_step);

                AVAILABLE_LB_PAIR_BIN_STEPS.save(
                    deps.storage,
                    (token_a.unique_key(), token_b.unique_key()),
                    &bin_step_list,
                )?;

                ephemeral_storage_w(deps.storage).remove();
                Ok(Response::default()
                    .set_data(to_binary(&lb_pair)?)
                    .add_attribute("lb_pair_address", lb_pair.contract.address)
                    .add_attribute("lb_pair_hash", lb_pair.contract.code_hash))
            }
            None => Err(StdError::generic_err("Expecting contract id")),
        },
        _ => Err(StdError::generic_err("Unknown reply id")),
    }
}
