use crate::{Error, Result};
use cosmwasm_std::{Deps, Uint256};
use liquidity_book::interfaces::lb_token2::*;

pub fn query_name(deps: Deps) -> Result<NameResponse> {
    // Implement name query logic
    Ok(NameResponse {
        name: "TokenName".to_string(),
    })
}

pub fn query_symbol(deps: Deps) -> Result<SymbolResponse> {
    // Implement symbol query logic
    Ok(SymbolResponse {
        symbol: "TKN".to_string(),
    })
}

pub fn query_total_supply(deps: Deps, id: u32) -> Result<TotalSupplyResponse> {
    // Implement total supply query logic
    Ok(TotalSupplyResponse {
        total_supply: Uint256::from_u128(100u128),
    })
}

pub fn query_balance_of(deps: Deps, account: String, id: u32) -> Result<BalanceResponse> {
    // Implement balance query logic
    Ok(BalanceResponse {
        balance: Uint256::from(100u128),
    })
}

pub fn query_balance_of_batch(
    deps: Deps,
    accounts: Vec<String>,
    ids: Vec<u32>,
) -> Result<BalanceBatchResponse> {
    // Implement batch balance query logic
    Ok(BalanceBatchResponse {
        balances: vec![Uint256::from(100u128); accounts.len()],
    })
}

pub fn query_is_approved_for_all(
    deps: Deps,
    owner: String,
    spender: String,
) -> Result<ApprovalResponse> {
    // Implement approval query logic
    Ok(ApprovalResponse { approved: true })
}
