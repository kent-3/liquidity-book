use crate::prelude::*;
use crate::state::FACTORY;
use cosmwasm_std::{ContractInfo, Deps};
use lb_interfaces::lb_factory::{self, LbPairInformationResponse};
use lb_interfaces::lb_router::Version;
use shade_protocol::swap::core::TokenType;

/// Helper function to return the address of the LBPair
///
/// Revert if the pair is not created yet
pub fn _get_lb_pair_information(
    deps: Deps,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
    // _version: Version,
) -> Result<ContractInfo> {
    let factory = FACTORY.load(deps.storage)?;

    let msg = lb_factory::QueryMsg::GetLbPairInformation {
        token_x,
        token_y,
        bin_step,
    };

    let LbPairInformationResponse {
        lb_pair_information,
    } = deps.querier.query_wasm_smart::<LbPairInformationResponse>(
        factory.code_hash,
        factory.address,
        &msg,
    )?;

    Ok(lb_pair_information.lb_pair.contract)
}
