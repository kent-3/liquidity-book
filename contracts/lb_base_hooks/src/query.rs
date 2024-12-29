use crate::state::LB_PAIR;
use cosmwasm_std::{Deps, Env, StdResult};
use liquidity_book::{interfaces::lb_hooks::*, libraries::hooks};

pub fn get_lb_pair(deps: Deps) -> StdResult<GetLbPairResponse> {
    // let lb_pair = LB_PAIR.load(deps.storage)?.map(|lb_pair| lb_pair.0);
    let lb_pair = LB_PAIR.load(deps.storage)?;

    // TODO: let's see what happens when ILbPair gets serialized
    let response = GetLbPairResponse { lb_pair };

    Ok(response)
}

pub fn is_linked(deps: Deps, env: Env) -> StdResult<IsLinkedResponse> {
    let is_linked = if let Some(lb_pair) = LB_PAIR.load(deps.storage)? {
        let hooks = hooks::get_hooks(lb_pair.get_lb_hooks_parameters(deps.querier)?);

        hooks == deps.api.addr_canonicalize(env.contract.address.as_str())?
    } else {
        false
    };

    let response = IsLinkedResponse { is_linked };

    Ok(response)
}
