use crate::{prelude::*, state::FACTORY};
use cosmwasm_std::{Addr, ContractInfo, Deps, Uint128};
use lb_interfaces::{lb_factory::ILbFactory, lb_router::Version};

// NOTE: We are following the joe-v2 versioning, starting from V2_2.

/// Helper function to return the address of the LBPair
///
/// Revert if the pair is not created yet
pub fn _get_lb_pair_information(
    deps: Deps,
    token_x: ContractInfo,
    token_y: ContractInfo,
    bin_step: u16,
    version: Version,
) -> Result<ContractInfo> {
    if version == Version::V2 {
        unimplemented!()
    } else if version == Version::V2_1 {
        unimplemented!()
    } else {
        let factory = FACTORY.load(deps.storage)?;

        let lb_pair_information = ILbFactory(factory).get_lb_pair_information(
            deps.querier,
            token_x,
            token_y,
            bin_step,
        )?;

        // let msg = lb_factory::QueryMsg::GetLbPairInformation {
        //     token_x,
        //     token_y,
        //     bin_step,
        // };
        //
        // let LbPairInformationResponse {
        //     lb_pair_information,
        // } = deps.querier.query_wasm_smart::<LbPairInformationResponse>(
        //     factory.code_hash,
        //     factory.address,
        //     &msg,
        // )?;

        Ok(lb_pair_information.lb_pair.contract)
    }

    // Err(Error::PairNotCreated {
    //     token_x: token_x.address().to_string(),
    //     token_y: token_y.address().to_string(),
    //     bin_step,
    // })
}

pub fn _get_pair(
    deps: Deps,
    token_x: ContractInfo,
    token_y: ContractInfo,
    bin_step: u16,
    version: Version,
) -> Result<ContractInfo> {
    if version == Version::V1 {
        unimplemented!()
    } else {
        _get_lb_pair_information(deps, token_x, token_y, bin_step, version)
    }
}

pub fn _get_pairs(
    deps: Deps,
    pair_bin_steps: Vec<u16>,
    versions: Vec<Version>,
    token_path: Vec<ContractInfo>, // contracts that implements the snip20 interface
) -> Result<Vec<ContractInfo>> {
    let mut pairs: Vec<ContractInfo> = Vec::with_capacity(pair_bin_steps.len());

    #[allow(unused_assignments)]
    let mut token = ContractInfo {
        address: Addr::unchecked(""),
        code_hash: "".to_string(),
    };
    let mut token_next = token_path[0].clone();

    for i in 0..pairs.len() {
        token = token_next;
        token_next = token_path[i + 1].clone();

        pairs[i] = _get_pair(
            deps,
            token.clone(),
            token_next.clone(),
            pair_bin_steps[i].clone(),
            versions[i].clone(),
        )?;
    }

    Ok(pairs)
}

// TODO: The point of this function is to transfer tokens from the user to the lb-pair contract.
// HOWEVER, we should make the swap functions be invoked by the snip20 receive message.
pub fn _safe_transfer_from(
    token: ContractInfo,
    from: Addr,
    to: Addr,
    amount: Uint128,
) -> Result<()> {
    if amount == Uint128::zero() {
        return Ok(());
    }
    todo!()
}
