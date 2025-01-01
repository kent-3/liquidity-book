use crate::state::LB_PAIR;
use cosmwasm_std::{Deps, Env, StdResult};
use liquidity_book::interfaces::lb_hooks::*;

// TODO: is it alright to use StdResult instead of crate::Result? there's really no need for a
// custom error type here, but it's breaking convention

/// Returns the LBPair contract.
pub fn get_lb_pair(deps: Deps) -> StdResult<GetLbPairResponse> {
    // let lb_pair = LB_PAIR.load(deps.storage)?.map(|lb_pair| lb_pair.0);
    let lb_pair = LB_PAIR.load(deps.storage)?;

    // TODO: let's see what happens when ILbPair gets serialized
    Ok(GetLbPairResponse { lb_pair })
}

/// Checks if the contract is linked to the pair.
pub fn is_linked(deps: Deps, env: Env) -> StdResult<IsLinkedResponse> {
    let is_linked = if let Some(lb_pair) = LB_PAIR.load(deps.storage)? {
        if let Some(hooks) = lb_pair.get_lb_hooks_parameters(deps.querier)? {
            hooks.address == env.contract.address
        } else {
            false
        }
    } else {
        false
    };

    Ok(IsLinkedResponse { is_linked })
}
