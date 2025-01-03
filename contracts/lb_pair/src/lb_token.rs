// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_token2::LbTokenError as Error;

/// Alias for Result<T, LbTokenError>
pub type Result<T, E = Error> = core::result::Result<T, E>;

use cosmwasm_std::{Addr, Deps, DepsMut, Env, Event, MessageInfo, Response, Uint256};
use liquidity_book::interfaces::lb_token2::*;
use secret_toolkit::storage::Keymap;

// NOTE: use address as suffix to create nested Keymaps for BALANCES and SPENDER_APPROVALS.

/// The mapping from account to token id to account balance.
pub(crate) static BALANCES: Keymap<u32, Uint256> = Keymap::new(b"balances");

/// The mapping from token id to total supply.
pub(crate) static TOTAL_SUPPLIES: Keymap<u32, Uint256> = Keymap::new(b"total_supplies");

/// Mapping from account to spender approvals.
pub(crate) static SPENDER_APPROVALS: Keymap<String, bool> = Keymap::new(b"spender_approvals");

/// Modifier to check if the spender is approved for all.
pub fn check_approval(deps: Deps, from: String, spender: String) -> Result<()> {
    if !_is_approved_for_all(deps, &from, &spender) {
        Err(Error::SpenderNotApproved { from, spender })
    } else {
        Ok(())
    }
}

// TODO: once again, what to do about address zero?
/// Modifier to check if the address is not zero or the contract itself.
pub fn not_address_zero_or_this(env: Env, account: String) -> Result<()> {
    _not_address_zero_or_this(env, account)
}

/// Modifier to check if the length of the arrays are equal.
pub fn check_length(length_a: usize, length_b: usize) -> Result<()> {
    _check_length(length_a, length_b)
}

/// Returns the name of the token.
pub fn name() -> Result<NameResponse> {
    Ok(NameResponse {
        name: "Liquidity Book Token".to_string(),
    })
}

/// Returns the symbol of the token, usually a shorter version of the name.
pub fn symbol() -> Result<SymbolResponse> {
    Ok(SymbolResponse {
        symbol: "LBT".to_string(),
    })
}

/// Returns the total supply of token of type `id`.
pub fn total_supply(deps: Deps, id: u32) -> Result<TotalSupplyResponse> {
    Ok(TotalSupplyResponse {
        total_supply: TOTAL_SUPPLIES.get(deps.storage, &id).unwrap_or_default(),
    })
}

// TODO: viewing keys

/// Returns the amount of tokens of type `id` owned by `account`.
pub fn balance_of(deps: Deps, account: String, id: u32) -> Result<BalanceResponse> {
    Ok(BalanceResponse {
        balance: BALANCES
            .add_suffix(account.as_bytes())
            .get(deps.storage, &id)
            .unwrap_or_default(),
    })
}

/// Return the balance of multiple (account/id) pairs.
pub fn balance_of_batch(
    deps: Deps,
    accounts: Vec<String>,
    ids: Vec<u32>,
) -> Result<BalanceBatchResponse> {
    // Implement batch balance query logic
    check_length(accounts.len(), ids.len())?;

    let mut batch_balances = Vec::with_capacity(accounts.len());

    for i in 0..accounts.len() {
        batch_balances[i] = balance_of(deps, accounts[i].clone(), ids[i])?.balance
    }

    Ok(BalanceBatchResponse {
        balances: batch_balances,
    })
}

/// Returns true if `spender` is approved to transfer `owner`'s tokens or if `spender` is the `owner`.
pub fn is_approved_for_all(deps: Deps, owner: String, spender: String) -> Result<ApprovalResponse> {
    Ok(ApprovalResponse {
        approved: _is_approved_for_all(deps, &owner, &spender),
    })
}

/// Grants or revokes permission to `spender` to transfer the caller's tokens, according to `approved`.
pub fn approve_for_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: String,
    approved: bool,
) -> Result<Response> {
    _approve_for_all(deps, info.sender.to_string(), spender, approved)
}

// NOTE: this is overridden by the function of the same name in crate::execute to include the hooks
pub fn batch_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: String,
    to: String,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> Result<Response> {
    check_approval(deps.as_ref(), from.clone(), info.sender.to_string())?;

    let from = deps.api.addr_validate(&from)?;
    let to = deps.api.addr_validate(&to)?;

    _batch_transfer_from(deps, env, info, from, to, ids, amounts)
}

pub(crate) fn _is_approved_for_all(deps: Deps, owner: &String, spender: &String) -> bool {
    // return owner == spender || _spenderApprovals[owner][spender];

    owner == spender
        || SPENDER_APPROVALS
            .add_suffix(owner.as_bytes())
            .get(deps.storage, spender)
            .unwrap_or_default()
}

/// Mint `amount` of `id` to `account`.
/// The `account` must not be the zero address.
/// The event should be emitted by the contract that inherits this contract.
pub(crate) fn _mint(deps: &mut DepsMut, account: Addr, id: u32, amount: Uint256) -> Result<()> {
    // TODO: Is there really no way to update the values in-place? I have to get, mutate, insert.

    let mut bin_total_supply = TOTAL_SUPPLIES.get(deps.storage, &id).unwrap_or_default();

    bin_total_supply += amount;

    TOTAL_SUPPLIES.insert(deps.storage, &id, &bin_total_supply)?;

    let mut balance = BALANCES
        .add_suffix(account.as_bytes())
        .get(deps.storage, &id)
        .unwrap_or_default();

    balance += amount;

    BALANCES
        .add_suffix(account.as_bytes())
        .insert(deps.storage, &id, &balance)?;

    Ok(())

    // Original:
    //
    // _totalSupplies[id] += amount;
    //
    // TODO: why is this part unchecked? what's that do?
    // unchecked {
    //     _balances[account][id] += amount;
    // }
}

/// Burn `amount` of `id` from `account`.
/// The `account` must not be the zero address.
/// The event should be emitted by the contract that inherits this contract.
pub(crate) fn _burn(deps: &mut DepsMut, account: Addr, id: u32, amount: Uint256) -> Result<()> {
    let account_balances = BALANCES.add_suffix(account.as_bytes());

    let balance = account_balances.get(deps.storage, &id).unwrap_or_default();

    if balance < amount {
        return Err(Error::BurnExceedsBalance {
            account,
            id,
            amount,
        });
    }

    let mut bin_total_supply = TOTAL_SUPPLIES
        .get(deps.storage, &id)
        .expect("attempting to burn when total supply is zero");

    bin_total_supply -= amount;

    TOTAL_SUPPLIES.insert(deps.storage, &id, &bin_total_supply)?;

    account_balances.insert(deps.storage, &id, &(balance - amount))?;

    Ok(())

    // Original:
    //
    // uint256 balance = accountBalances[id];
    // if (balance < amount) revert LBToken__BurnExceedsBalance(account, id, amount);
    //
    // unchecked {
    //     _totalSupplies[id] -= amount;
    //     accountBalances[id] = balance - amount;
    // }
}

pub(crate) fn _batch_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    to: Addr,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> Result<Response> {
    check_length(ids.len(), amounts.len())?;
    not_address_zero_or_this(env, to.to_string())?;

    let from_balances = BALANCES.add_suffix(from.as_bytes());
    let to_balances = BALANCES.add_suffix(to.as_bytes());

    for i in 0..ids.len() {
        let id = ids[i];
        let amount = amounts[i];

        let from_balance = from_balances.get(deps.storage, &id).unwrap_or_default();
        if from_balance < amount {
            return Err(Error::TransferExceedsBalance { from, id, amount });
        }

        from_balances.insert(deps.storage, &id, &(from_balance - amount))?;

        let to_balance = to_balances.get(deps.storage, &id).unwrap_or_default();
        to_balances.insert(deps.storage, &id, &(to_balance + amount))?;
    }

    let event = Event::transfer_batch(info.sender, from.to_string(), to.to_string(), ids, amounts);

    Ok(Response::new().add_event(event))

    // Original:
    //
    // mapping(uint256 => uint256) storage fromBalances = _balances[from];
    // mapping(uint256 => uint256) storage toBalances = _balances[to];
    //
    // for (uint256 i; i < ids.length;) {
    //     uint256 id = ids[i];
    //     uint256 amount = amounts[i];
    //
    //     uint256 fromBalance = fromBalances[id];
    //     if (fromBalance < amount) revert LBToken__TransferExceedsBalance(from, id, amount);
    //
    //     unchecked {
    //         fromBalances[id] = fromBalance - amount;
    //         toBalances[id] += amount;
    //
    //         ++i;
    //     }
    // }
    //
    // emit TransferBatch(msg.sender, from, to, ids, amounts);
}

pub(crate) fn _approve_for_all(
    deps: DepsMut,
    owner: String,
    spender: String,
    approved: bool,
) -> Result<Response> {
    if owner == spender {
        return Err(Error::SelfApproval(owner));
    }

    SPENDER_APPROVALS
        .add_suffix(owner.as_bytes())
        .insert(deps.storage, &spender, &approved)?;

    let event = Event::approval_for_all(owner, spender, approved);

    Ok(Response::new().add_event(event))
}

pub(crate) fn _not_address_zero_or_this(env: Env, account: String) -> Result<()> {
    if account == "" || account == env.contract.address.to_string() {
        Err(Error::AddressThisOrZero)
    } else {
        Ok(())
    }
}

pub(crate) fn _check_length(length_a: usize, length_b: usize) -> Result<()> {
    if length_a != length_b {
        Err(Error::InvalidLength)
    } else {
        Ok(())
    }
}
