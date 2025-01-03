mod contract;
mod execute;
mod query;
mod state;

// Use this crate's custom Error type
pub use liquidity_book::interfaces::lb_token2::LbTokenError as Error;

/// Alias for Result<T, LbPairError>
pub type Result<T, E = Error> = core::result::Result<T, E>;

// /**
//  * @dev Modifier to check if the spender is approved for all.
//  */
// modifier checkApproval(address from, address spender) {
//     if (!_isApprovedForAll(from, spender)) revert LBToken__SpenderNotApproved(from, spender);
//     _;
// }
//
// /**
//  * @dev Modifier to check if the address is not zero or the contract itself.
//  */
// modifier notAddressZeroOrThis(address account) {
//     _notAddressZeroOrThis(account);
//     _;
// }
//
// /**
//  * @dev Modifier to check if the length of the arrays are equal.
//  */
// modifier checkLength(uint256 lengthA, uint256 lengthB) {
//     _checkLength(lengthA, lengthB);
//     _;
// }
