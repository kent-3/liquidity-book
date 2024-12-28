use crate::state::LB_PAIR;
use cosmwasm_std::{Deps, StdResult};
use lb_interfaces::lb_hooks::*;

pub fn get_lb_pair(deps: Deps) -> StdResult<GetLbPairResponse> {
    let lb_pair = LB_PAIR.load(deps.storage)?;

    let response = GetLbPairResponse { lb_pair };

    Ok(response)
}

pub fn is_linked(deps: Deps) -> StdResult<IsLinkedResponse> {
    let lb_pair = LB_PAIR.load(deps.storage)?;
    let is_linked = lb_pair.is_some();

    let response = IsLinkedResponse { is_linked };

    Ok(response)
}
