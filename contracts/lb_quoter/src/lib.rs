use cosmwasm_std::{
    entry_point, to_binary, Binary, ContractInfo, Deps, DepsMut, Env, MessageInfo, StdError,
    Uint128,
};
use lb_interfaces::{
    lb_factory::ILbFactory,
    lb_pair::{ILbPair, LbPairInformation},
    lb_quoter::{
        ExecuteMsg, FactoryV2_2Response, InstantiateMsg, QueryMsg, Quote, RouterV2_2Response,
    },
    lb_router::{self, ILbRouter},
};
use lb_libraries::{
    constants::SCALE_OFFSET,
    math::{
        u128x128_math::U128x128MathError,
        u256x256_math::{U256x256Math, U256x256MathError},
    },
    price_helper::PriceHelper,
};
use shade_protocol::secret_storage_plus::Item;

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub const FACTORY_V2_2: Item<Option<ContractInfo>> = Item::new("factory_v2_2");
pub const ROUTER_V2_2: Item<Option<ContractInfo>> = Item::new("router_v2_2");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Generic Errors
    #[error("Generic {0}")]
    Generic(String),

    #[error("InvalidLength")]
    InvalidLength,

    // Error Wrappings from Dependencies
    #[error(transparent)]
    CwErr(#[from] StdError),
    #[error(transparent)]
    U128Err(#[from] U128x128MathError),
    #[error(transparent)]
    U256Err(#[from] U256x256MathError),
}

#[entry_point]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> Result<()> {
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

    Ok(())
}

// TODO: see what happens if I remove this
#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> Result<()> {
    unimplemented!()
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary> {
    match msg {
        QueryMsg::GetFactoryV2_2 {} => {
            let factory_v2_2 = FACTORY_V2_2.load(deps.storage)?;
            let response = FactoryV2_2Response { factory_v2_2 };
            to_binary(&response)
        }
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

pub fn find_best_path_from_amount_in(
    deps: Deps,
    route: Vec<ContractInfo>,
    amount_in: Uint128,
) -> Result<Quote> {
    if route.len() < 2 {
        return Err(Error::InvalidLength);
    }

    let mut quote = Quote::default();

    quote.route = route.clone();

    let swap_length = route.len() - 1;
    quote.pairs = Vec::with_capacity(swap_length);
    quote.bin_steps = Vec::with_capacity(swap_length);
    quote.versions = Vec::with_capacity(swap_length);
    quote.fees = Vec::with_capacity(swap_length);
    quote.amounts = Vec::with_capacity(swap_length);
    quote.virtual_amounts_without_slippage = Vec::with_capacity(route.len());

    quote.amounts[0] = amount_in;
    quote.virtual_amounts_without_slippage[0] = amount_in;

    for i in 0..swap_length {
        if let Some(factory) = FACTORY_V2_2.load(deps.storage)? {
            // Fetch swaps for V2.2
            let lb_pairs_available: Vec<LbPairInformation> =
                ILbFactory(factory).get_all_lb_pairs(deps.querier, &route[i], &route[i + 1])?;

            if lb_pairs_available.len() > 0 && quote.amounts[i] > Uint128::zero() {
                for j in 0..lb_pairs_available.len() {
                    if !lb_pairs_available[j].ignored_for_routing {
                        let lb_pair = ILbPair(lb_pairs_available[j].clone().lb_pair.contract);

                        let swap_for_y = lb_pair.get_token_y(deps.querier)? == route[i + 1];

                        let lb_router::SwapOutResponse {
                            amount_in_left,
                            amount_out: swap_amount_out,
                            fee: fees,
                        } = ILbRouter(ROUTER_V2_2.load(deps.storage)?.unwrap()).get_swap_out(
                            deps.querier,
                            lb_pair.0.clone(),
                            quote.amounts[i],
                            swap_for_y,
                        )?;

                        if amount_in_left == Uint128::zero()
                            && swap_amount_out > quote.amounts[i + 1]
                        {
                            quote.amounts[i + 1] = swap_amount_out;
                            quote.pairs[i] = lb_pair.0.clone();
                            quote.bin_steps[i] = lb_pairs_available[j].bin_step;
                            quote.versions[i] = lb_router::Version::V2_2;

                            //  // Getting current price
                            let active_id = lb_pair.get_active_id(deps.querier)?;
                            quote.virtual_amounts_without_slippage[i + 1] = _get_v2_quote(
                                quote.virtual_amounts_without_slippage[i] - fees,
                                active_id,
                                quote.bin_steps[i],
                                swap_for_y,
                            )?;

                            // TODO: double check this math is OK
                            quote.fees[i] = fees.multiply_ratio(10u128.pow(18), quote.amounts[i]);
                            // fee percentage in amountIn
                        }
                    }
                }
            }
        }

        // NOTE: Future versions of this router can add more blocks like this:

        // if let Some(factory) = FACTORY_V2_3.load(deps.storage)? {
        //     todo!()
        // }
    }

    Ok(quote)
}

pub fn find_best_path_from_amount_out(
    deps: Deps,
    route: Vec<ContractInfo>,
    amount_out: Uint128,
) -> Result<Quote> {
    todo!()
}

// NOTE: We are following the joe-v2 versioning, starting from V2_2.

pub fn _get_v2_quote(
    amount: Uint128,
    active_id: u32,
    bin_step: u16,
    swap_for_y: bool,
) -> Result<Uint128> {
    if swap_for_y {
        let x = PriceHelper::get_price_from_id(active_id, bin_step)?;
        let y = ethnum::U256::new(amount.u128());
        let quote = U256x256Math::mul_shift_round_down(x, y, SCALE_OFFSET)?.as_u128();

        Ok(Uint128::from(quote))
    } else {
        let x = ethnum::U256::new(amount.u128());
        let y = PriceHelper::get_price_from_id(active_id, bin_step)?;
        let quote = U256x256Math::shift_div_round_down(x, SCALE_OFFSET, y)?.as_u128();

        Ok(Uint128::from(quote))
    }
}
