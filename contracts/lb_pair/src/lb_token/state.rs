use cosmwasm_std::{Addr, Uint256};
use secret_toolkit::storage::Keymap;

// NOTE: use address as suffix to create nested Keymaps for BALANCES and SPENDER_APPROVALS.

/// The mapping from account to token id to account balance.
pub(crate) static BALANCES: Keymap<u32, Uint256> = Keymap::new(b"balances");

/// The mapping from token id to total supply.
pub(crate) static TOTAL_SUPPLIES: Keymap<u32, Uint256> = Keymap::new(b"total_supplies");

/// Mapping from account to spender approvals.
pub(crate) static SPENDER_APPROVALS: Keymap<Addr, bool> = Keymap::new(b"spender_approvals");
