//! ### Custom Errors for LB_Router contract.

#![allow(unused)] // For beginning only.

use cosmwasm_std::Uint128;

// TODO: there are some new error types in LBRouter V2.1

#[derive(thiserror::Error, Debug)]
pub enum LBRouterError {
    #[error("The sender is not WNATIVE")]
    SenderIsNotWNATIVE,

    #[error("Pair not created: {token_x}, {token_y}, bin step: {bin_step}")]
    PairNotCreated {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },

    #[error("Wrong amounts. Amount: {amount}, Reserve: {reserve}")]
    WrongAmounts { amount: Uint128, reserve: Uint128 },

    #[error("Swap overflows for bin id {id}")]
    SwapOverflows { id: u32 },

    #[error("Broken swap safety check")]
    BrokenSwapSafetyCheck,

    #[error("Not factory owner")]
    NotFactoryOwner,

    #[error("Too many tokens in. Excess: {excess}")]
    TooManyTokensIn { excess: Uint128 },

    #[error("Bin reserve overflows for bin id {id}")]
    BinReserveOverflows { id: Uint128 },

    #[error("Bin id overflows: {id}")]
    IdOverflows { id: Uint128 },

    #[error("Path lengths mismatch")]
    LengthsMismatch,

    #[error("Wrong token order")]
    WrongTokenOrder,

    #[error("{self:?}")]
    IdSlippageCaught {
        active_id_desired: u32,
        id_slippage: u32,
        active_id: u32,
    },

    #[error("{self:?}")]
    AmountSlippageCaught {
        amount_x_min: Uint128,
        amount_x: Uint128,
        amount_y_min: Uint128,
        amount_y: Uint128,
    },

    #[error("Id desired overflows. Id desired: {id_desired}, Id slippage: {id_slippage}")]
    IdDesiredOverflows { id_desired: u32, id_slippage: u32 },

    #[error("Failed to send WNATIVE to recipient {recipient}. Amount: {amount}")]
    FailedToSendNATIVE { recipient: String, amount: Uint128 },

    #[error("Deadline exceeded: {timestamp} > {deadline}")]
    DeadlineExceeded { deadline: u64, timestamp: u64 },

    #[error("Amount slippage BP too big. Amount slippage: {amount_slippage}")]
    AmountSlippageBpTooBig { amount_slippage: String },

    #[error("Insufficient amount out. Amount out min: {amount_out_min}, Amount out: {amount_out}")]
    InsufficientAmountOut {
        amount_out_min: Uint128,
        amount_out: Uint128,
    },

    #[error("Max amount in exceeded. Amount in max: {amount_in_max}, Amount in: {amount_in}")]
    MaxAmountInExceeded {
        amount_in_max: Uint128,
        amount_in: Uint128,
    },

    #[error("Invalid token path: {wrong_token}")]
    InvalidTokenPath { wrong_token: String },

    #[error("Invalid version: {0}")]
    InvalidVersion(u32),

    #[error("Wrong WNATIVE liquidity parameters. token_x: {token_x}, token_y: {token_y}, amount_x: {amount_x}, amount_y: {amount_y}, msg_value: {msg_value}")]
    WrongNativeLiquidityParameters {
        token_x: String,
        token_y: String,
        amount_x: Uint128,
        amount_y: Uint128,
        msg_value: Uint128,
    },

    // not in joe-v2
    #[error("Generic {0}")]
    Generic(String),

    #[error("Unknown reply id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Reply data is missing!")]
    ReplyDataMissing,

    #[error(transparent)]
    CwErr(#[from] cosmwasm_std::StdError),

    #[error(transparent)]
    LbError(#[from] liquidity_book::libraries::Error),
    #[error(transparent)]
    PackedUint128MathError(
        #[from] liquidity_book::libraries::math::packed_u128_math::PackedUint128MathError,
    ),
}
