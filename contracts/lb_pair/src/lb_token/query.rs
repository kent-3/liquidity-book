use super::{
    check_length,
    state::{BALANCES, SPENDER_APPROVALS, TOTAL_SUPPLIES},
    Error, Result,
};
use cosmwasm_std::{Deps, Uint256};
use liquidity_book::interfaces::lb_token2::*;

/// Returns the name of the token.
pub fn query_name() -> Result<NameResponse> {
    Ok(NameResponse {
        name: "Liquidity Book Token".to_string(),
    })
}

/// Returns the symbol of the token, usually a shorter version of the name.
pub fn query_symbol() -> Result<SymbolResponse> {
    Ok(SymbolResponse {
        symbol: "LBT".to_string(),
    })
}

/// Returns the total supply of token of type `id`.
pub fn query_total_supply(deps: Deps, id: u32) -> Result<TotalSupplyResponse> {
    Ok(TotalSupplyResponse {
        total_supply: TOTAL_SUPPLIES.get(deps.storage, &id).unwrap_or_default(),
    })
}

// TODO: viewing keys

/// Returns the amount of tokens of type `id` owned by `account`.
pub fn query_balance_of(deps: Deps, account: String, id: u32) -> Result<BalanceResponse> {
    Ok(BalanceResponse {
        balance: BALANCES
            .add_suffix(account.as_bytes())
            .get(deps.storage, &id)
            .unwrap_or_default(),
    })
}

/// Return the balance of multiple (account/id) pairs.
pub fn query_balance_of_batch(
    deps: Deps,
    accounts: Vec<String>,
    ids: Vec<u32>,
) -> Result<BalanceBatchResponse> {
    // Implement batch balance query logic
    check_length(accounts.len(), ids.len())?;

    let mut batch_balances = Vec::with_capacity(accounts.len());

    for i in 0..accounts.len() {
        batch_balances[i] = query_balance_of(deps, accounts[i].clone(), ids[i])?.balance
    }

    Ok(BalanceBatchResponse {
        balances: batch_balances,
    })
}

/// Returns true if `spender` is approved to transfer `owner`'s tokens or if `spender` is the `owner`.
pub fn query_is_approved_for_all(
    deps: Deps,
    owner: String,
    spender: String,
) -> Result<ApprovalResponse> {
    Ok(ApprovalResponse {
        approved: _is_approved_for_all(deps, &owner, &spender),
    })
}

pub fn _is_approved_for_all(deps: Deps, owner: &String, spender: &String) -> bool {
    // return owner == spender || _spenderApprovals[owner][spender];

    owner == spender
        || SPENDER_APPROVALS
            .add_suffix(owner.as_bytes())
            .get(deps.storage, spender)
            .unwrap_or_default()
}
