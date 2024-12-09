use crate::{prelude::*, query::*, state::*};
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, ContractInfo, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, Uint256,
};
use lb_interfaces::{lb_factory, lb_pair, lb_quoter::*, lb_router};
use lb_libraries::lb_token::state_structs::LbPair;

/////////////// INSTANTIATE ///////////////
#[entry_point]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> Result<()> {
    let factory_v1 = msg
        .factory_v1
        .map(|raw_contract| raw_contract.valid(deps.api))
        .transpose()?;

    let router_v1 = msg
        .router_v1
        .map(|raw_contract| raw_contract.valid(deps.api))
        .transpose()?;

    let state = State {
        factory_v1,
        router_v1,
    };

    STATE.save(deps.storage, &state)?;

    Ok(())
}

/////////////// EXECUTE ///////////////
#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<()> {
    unimplemented!()
}

/////////////// QUERY ///////////////

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetFactoryV1 {} => {
            let factory_v1 = STATE.load(deps.storage)?.factory_v1;
            let response = FactoryV1Response { factory_v1 };
            to_binary(&response)
        }
        QueryMsg::GetRouterV1 {} => {
            let router_v1 = STATE.load(deps.storage)?.router_v1;
            let response = RouterV1Response { router_v1 };
            to_binary(&response)
        }
        QueryMsg::FindBestPathFromAmountIn { route, amount_in } => {
            to_binary(&find_best_path_from_amount_in(deps)?)
        }
        QueryMsg::FindBestPathFromAmountOut { route, amount_out } => {
            to_binary(&find_best_path_from_amount_out(deps)?)
        }
    }
    .map_err(Error::from)
}
