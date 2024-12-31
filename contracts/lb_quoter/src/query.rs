use crate::{
    helper::_get_v2_quote,
    prelude::{Error, Result},
    state::{FACTORY_V2_2, ROUTER_V2_2},
};
use cosmwasm_std::{Deps, Uint128};
use liquidity_book::interfaces::{
    lb_factory::ILbFactory,
    lb_pair::{ILbPair, LbPairInformation},
    lb_quoter::Quote,
    lb_router::{self, ILbRouter},
};
use shade_protocol::swap::core::TokenType;

pub fn find_best_path_from_amount_in(
    deps: Deps,
    route: Vec<TokenType>,
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
            let lb_pairs_available: Vec<LbPairInformation> = ILbFactory(factory).get_all_lb_pairs(
                deps.querier,
                route[i].clone(),
                route[i + 1].clone(),
            )?;

            if lb_pairs_available.len() > 0 && quote.amounts[i] > Uint128::zero() {
                for j in 0..lb_pairs_available.len() {
                    if !lb_pairs_available[j].ignored_for_routing {
                        let lb_pair = ILbPair(lb_pairs_available[j].clone().lb_pair.contract);

                        let swap_for_y = lb_pair.get_token_y(deps.querier)? == route[i + 1];

                        let lb_router::GetSwapOutResponse {
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
    route: Vec<TokenType>,
    amount_out: Uint128,
) -> Result<Quote> {
    todo!()
}
