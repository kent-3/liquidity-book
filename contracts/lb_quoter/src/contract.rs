use crate::{
    prelude::{Error, Result},
    query::{find_best_path_from_amount_in, find_best_path_from_amount_out},
    state::{FACTORY_V2_2, ROUTER_V2_2},
};
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use liquidity_book::interfaces::lb_quoter::{
    FactoryV2_2Response, InstantiateMsg, QueryMsg, RouterV2_2Response,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response> {
    let factory_v2_2 = msg
        .factory_v2_2
        .map(|raw_contract| raw_contract.valid(deps.api))
        .transpose()?;

    let router_v2_2 = msg
        .router_v2_2
        .map(|raw_contract| raw_contract.valid(deps.api))
        .transpose()?;

    FACTORY_V2_2.save(deps.storage, &factory_v2_2)?;
    ROUTER_V2_2.save(deps.storage, &router_v2_2)?;

    Ok(Response::new())
}

// TODO: see what happens if I remove this
// #[entry_point]
// pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<()> {
//     unimplemented!()
// }

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        // TODO: move to separate function for consistency
        QueryMsg::GetFactoryV2_2 {} => {
            let factory_v2_2 = FACTORY_V2_2.load(deps.storage)?;
            let response = FactoryV2_2Response { factory_v2_2 };
            to_binary(&response)
        }
        // TODO: move to separate function for consistency
        QueryMsg::GetRouterV2_2 {} => {
            let router_v2_2 = ROUTER_V2_2.load(deps.storage)?;
            let response = RouterV2_2Response { router_v2_2 };
            to_binary(&response)
        }
        QueryMsg::FindBestPathFromAmountIn { route, amount_in } => {
            to_binary(&find_best_path_from_amount_in(deps, route, amount_in)?)
        }
        QueryMsg::FindBestPathFromAmountOut { route, amount_out } => {
            to_binary(&find_best_path_from_amount_out(deps, route, amount_out)?)
        }
    }
    .map_err(Error::CwErr)
}
