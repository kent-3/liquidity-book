use ethnum::{serde::bytes::be, U256};
// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_token2::LbTokenError as Error;

/// Alias for Result<T, LbTokenError>
pub type Result<T, E = Error> = core::result::Result<T, E>;

use cosmwasm_std::{Addr, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult, Uint256};
use liquidity_book::{interfaces::lb_token2::*, Bytes32};
use secret_toolkit::storage::{Item, Keymap};

// TODO: There is no TOTAL total supply function... which kinda makes sense, since there are so
// many bins. It would cost more gas to always track the total supply, but maybe it's worth it?
// This total supply would represent the "total liquidity" or the largest amount that can be swapped.

// TODO: while we're at it, we could add even more state to track which bins each user has
// liquidity in, and each user's total liquidity (does that even mean anything)? Technically,
// we have the ability to iterate over the Keymap, though we should probably disable that for gas savings.
// I think we should keep it for Balances, disable it for total supplies. because each user's
// balances map is likely much smaller than the total supplies map. And if we keep a separate tally
// of total supply, we never need to iterate over total supplies.

// TODO: U256 is serialized as a "0x" prefixed hex strings by default. Figure out how to make it
// use bytes instead.
// They give this example in the docs, but I would need to create a wrapper struct I guess:
// #[derive(Deserialize, Serialize)]
// struct Example {
//     a: U256, // "0x2a"
//     #[serde(with = "ethnum::serde::decimal")]
//     b: I256, // "-42"
//     #[serde(with = "ethnum::serde::prefixed")]
//     c: U256, // "0x2a" or "42"
//     #[serde(with = "ethnum::serde::permissive")]
//     d: I256, // "-0x2a" or "-42" or -42
//     #[serde(with = "ethnum::serde::bytes::be")]
//     e: U256, // [0x2a, 0x00, ..., 0x00]
//     #[serde(with = "ethnum::serde::bytes::le")]
//     f: I256, // [0xd6, 0xff, ..., 0xff]
//     #[serde(with = "ethnum::serde::compressed_bytes::be")]
//     g: U256, // [0x2a]
//     #[serde(with = "ethnum::serde::compressed_bytes::le")]
//     h: I256, // [0xd6]
// }

// TODO: OR just use Uint256 instead?

// NOTE: use address as suffix to create nested Keymaps for BALANCES and SPENDER_APPROVALS.

// TODO: exploring this idea
// TODO: I think this needs to be initialized during contract instantation, or the update function
// will fail.
/// global total supply tracker
pub(crate) static TOTAL_SUPPLY: Item<Uint256> = Item::new(b"total_supply");

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

// TODO: flatten these query responses?

/// Returns the name of the token.
pub fn name() -> String {
    // Ok(NameResponse {
    //     name: "Liquidity Book Token".to_string(),
    // })

    "Liquidity Book Token".to_string()
}

/// Returns the symbol of the token, usually a shorter version of the name.
pub fn symbol() -> String {
    // Ok(SymbolResponse {
    //     symbol: "LBT".to_string(),
    // })

    "LBT".to_string()
}

/// Returns the total supply of token of type `id`.
pub fn total_supply(deps: Deps, id: u32) -> Uint256 {
    // TODO: decide

    // Ok(TotalSupplyResponse {
    //     total_supply: TOTAL_SUPPLIES.get(deps.storage, &id).unwrap_or_default(),
    // })

    // U256::from_be_bytes(TOTAL_SUPPLIES.get(deps.storage, &id).unwrap_or_default())

    TOTAL_SUPPLIES.get(deps.storage, &id).unwrap_or_default()
}

// TODO: viewing keys

/// Returns the amount of tokens of type `id` owned by `account`.
pub fn balance_of(deps: Deps, account: String, id: u32) -> Uint256 {
    // Ok(BalanceResponse {
    //     balance: BALANCES
    //         .add_suffix(account.as_bytes())
    //         .get(deps.storage, &id)
    //         .unwrap_or_default(),
    // })

    BALANCES
        .add_suffix(account.as_bytes())
        .get(deps.storage, &id)
        .unwrap_or_default()
}

/// Return the balance of multiple (account/id) pairs.
pub fn balance_of_batch(deps: Deps, accounts: Vec<String>, ids: Vec<u32>) -> Result<Vec<Uint256>> {
    check_length(accounts.len(), ids.len())?;

    let mut batch_balances = Vec::with_capacity(accounts.len());

    for i in 0..accounts.len() {
        batch_balances[i] = balance_of(deps, accounts[i].clone(), ids[i])
    }

    // Ok(BalanceBatchResponse {
    //     balances: batch_balances,
    // })

    Ok(batch_balances)
}

/// Returns true if `spender` is approved to transfer `owner`'s tokens or if `spender` is the `owner`.
pub fn is_approved_for_all(deps: Deps, owner: String, spender: String) -> bool {
    // Ok(ApprovalResponse {
    //     approved: _is_approved_for_all(deps, &owner, &spender),
    // })

    _is_approved_for_all(deps, &owner, &spender)
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

/// Batch transfers `amounts` of `ids` from `from` to `to`.
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

/// Returns true if `spender` is approved to transfer `owner`'s tokens or if `spender` is the `owner`.
pub(crate) fn _is_approved_for_all(deps: Deps, owner: &String, spender: &String) -> bool {
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

    // TODO: potentially
    TOTAL_SUPPLY.update(deps.storage, |mut total_supply| -> StdResult<_> {
        total_supply += amount;
        Ok(total_supply)
    })?;

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

    // TODO: potentially
    TOTAL_SUPPLY.update(deps.storage, |mut total_supply| -> StdResult<_> {
        total_supply -= amount;
        Ok(total_supply)
    })?;

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
