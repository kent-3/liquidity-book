//! ### Custom Errors for lb-factory contract.

use std::string::FromUtf8Error;

use cosmwasm_std::{Addr, StdError};
use liquidity_book::libraries::{
    bin_helper::BinError,
    fee_helper::FeeError,
    math::{
        liquidity_configurations::LiquidityConfigurationsError, u128x128_math::U128x128MathError,
        u256x256_math::U256x256MathError,
    },
    oracle_helper::OracleError,
    pair_parameter_helper::PairParametersError,
};

#[derive(thiserror::Error, Debug)]
pub enum LbFactoryError {
    #[error("Tokens are identical! Both addresses are {token}!")]
    IdenticalAddresses { token: String },

    #[error("Quote Asset {quote_asset} is not whitelisted!")]
    QuoteAssetNotWhitelisted { quote_asset: String },

    #[error("Quote Asset {quote_asset} is already whitelisted!")]
    QuoteAssetAlreadyWhitelisted { quote_asset: String },

    // Not sure if applicable
    #[error("Address zero!")]
    AddressZero,

    #[error("LbPair ({token_x}, {token_y}, bin_step: {bin_step}) already exists!")]
    LbPairAlreadyExists {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },

    #[error("LbPair ({token_x}, {token_y}, bin_step: {bin_step}) does not exist!")]
    LbPairDoesNotExist {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },

    #[error("LbPair ({token_x}, {token_y}, bin_step: {bin_step}) not created!")]
    LbPairNotCreated {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },

    #[error("Flash Loan Fee above max: {fee} > {max_fee}!")]
    FlashLoanFeeAboveMax { fee: u8, max_fee: u8 },

    #[error("Bin step {bin_step} is too low!")]
    BinStepTooLow { bin_step: u16 },

    #[error("Preset {bin_step} is locked for users! {user} is not the owner!")]
    PresetIsLockedForUsers { user: Addr, bin_step: u16 },

    #[error("LbPair.ignored is already in the same state!")]
    LbPairIgnoredIsAlreadyInTheSameState,

    #[error("Bin step {bin_step} has no preset!")]
    BinStepHasNoPreset { bin_step: u16 },

    #[error("Preset open state is already in the same state!")]
    PresetOpenStateIsAlreadyInTheSameState,

    #[error("Fee recipient is already {fee_recipient}!")]
    SameFeeRecipient { fee_recipient: Addr },

    #[error("Flash loan fee is already {fee}!")]
    SameFlashLoanFee { fee: u8 },

    #[error(
        "LbPair safety check failed. {lb_pair_implementation} factory address does not match this one!"
    )]
    LbPairSafetyCheckFailed { lb_pair_implementation: Addr },

    #[error("Lb implementation is already set to code ID {implementation}!")]
    SameImplementation { implementation: u64 },

    #[error("The LbPair implementation has not been set yet!")]
    ImplementationNotSet,

    // not in joe-v2
    #[error("{0}!")]
    Generic(String),
    #[error("Only the Owner can do that!")]
    OnlyOwner,
    #[error("Transaction is blocked by contract status")]
    TransactionBlock(),
    #[error("Unknown reply id: {id}")]
    UnknownReplyId { id: u64 },
    #[error("Reply data is missing!")]
    ReplyDataMissing,

    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    CwErr(#[from] StdError),
    #[error(transparent)]
    BinErr(#[from] BinError),
    #[error(transparent)]
    FeeErr(#[from] FeeError),
    #[error(transparent)]
    OracleErr(#[from] OracleError),
    #[error(transparent)]
    ParamsErr(#[from] PairParametersError),
    #[error(transparent)]
    LiquidityConfigErr(#[from] LiquidityConfigurationsError),
    #[error(transparent)]
    U128Err(#[from] U128x128MathError),
    #[error(transparent)]
    U256Err(#[from] U256x256MathError),
}
