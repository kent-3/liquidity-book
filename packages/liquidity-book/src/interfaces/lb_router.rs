use super::lb_pair::LbPair;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    Addr, Binary, ContractInfo, QuerierWrapper, StdResult, Uint128, Uint256, Uint64,
};
// TODO: replace these dependencies
use shade_protocol::contract_interfaces::swap::core::TokenType;
use shade_protocol::utils::asset::RawContract;

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
    LbError(#[from] crate::libraries::Error),
    #[error(transparent)]
    PackedUint128MathError(
        #[from] crate::libraries::math::packed_u128_math::PackedUint128MathError,
    ),
}

// TODO: decide about the Version stuff. It is very specific to Trader Joe, but we could use this
// approach to support swaps from other DEXs. For example: V1 = shade_swap, V2 = liquidity_book.
// Then a path could contain a mix of pair types.

/// This enum represents the version of the pair requested
/// - V1: Joe V1 pair
/// - V2: LB pair V2. Also called legacyPair
/// - V2_1: LB pair V2.1
/// - V2_2: LB pair V2.2 (current version)
#[cw_serde]
pub enum Version {
    V1,
    V2,
    V2_1,
    V2_2,
}

/// The liquidity parameters, such as:
/// - tokenX: The address of token X
/// - tokenY: The address of token Y
/// - binStep: The bin step of the pair
/// - amountX: The amount to send of token X
/// - amountY: The amount to send of token Y
/// - amountXMin: The min amount of token X added to liquidity
/// - amountYMin: The min amount of token Y added to liquidity
/// - activeIdDesired: The active id that user wants to add liquidity from
/// - idSlippage: The number of id that are allowed to slip
/// - deltaIds: The list of delta ids to add liquidity (`deltaId = activeId - desiredId`)
/// - distributionX: The distribution of tokenX with sum(distributionX) = 1e18 (100%) or 0 (0%)
/// - distributionY: The distribution of tokenY with sum(distributionY) = 1e18 (100%) or 0 (0%)
/// - to: The address of the recipient
/// - refundTo: The address of the recipient of the refunded tokens if too much tokens are sent
/// - deadline: The deadline of the transaction
#[cw_serde]
pub struct LiquidityParameters {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub amount_x: Uint128,
    pub amount_y: Uint128,
    pub amount_x_min: Uint128,
    pub amount_y_min: Uint128,
    pub active_id_desired: u32,
    pub id_slippage: u32,
    pub delta_ids: Vec<i64>,
    pub distribution_x: Vec<Uint64>,
    pub distribution_y: Vec<Uint64>,
    pub to: String,
    pub refund_to: String,
    pub deadline: Uint64,
}

/// The path parameters, such as:
/// - pairBinSteps: The list of bin steps of the pairs to go through
/// - versions: The list of versions of the pairs to go through
/// - tokenPath: The list of tokens in the path to go through
#[cw_serde]
pub struct Path {
    pub pair_bin_steps: Vec<u16>,
    pub versions: Vec<Version>,
    pub token_path: Vec<TokenType>,
}

/// A thin wrapper around `ContractInfo` that provides additional
/// methods to interact with an LB Router contract.
pub struct ILbRouter(pub ContractInfo);

impl std::ops::Deref for ILbRouter {
    type Target = ContractInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: add the ExecuteMsg constructor methods
impl ILbRouter {
    pub fn get_factory(&self, querier: QuerierWrapper) -> StdResult<GetFactoryResponse> {
        querier.query_wasm_smart::<GetFactoryResponse>(
            self.0.code_hash.clone(),
            self.0.address.clone(),
            &QueryMsg::GetFactory {},
        )
    }

    pub fn get_id_from_price(
        &self,
        querier: QuerierWrapper,
        lb_pair: ContractInfo,
        price: Uint256,
    ) -> StdResult<u32> {
        querier
            .query_wasm_smart::<GetIdFromPriceResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetIdFromPrice { lb_pair, price },
            )
            .map(|response| response.id)
    }

    /// remember that price is a 128x128 fixed point number represented by a Uint256
    pub fn get_price_from_id(
        &self,
        querier: QuerierWrapper,
        lb_pair: ContractInfo,
        id: u32,
    ) -> StdResult<Uint256> {
        querier
            .query_wasm_smart::<GetPriceFromIdResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetPriceFromId { lb_pair, id },
            )
            .map(|response| response.price)
    }

    pub fn get_swap_in(
        &self,
        querier: QuerierWrapper,
        lb_pair: ContractInfo,
        amount_out: Uint128,
        swap_for_y: bool,
    ) -> StdResult<GetSwapInResponse> {
        querier.query_wasm_smart::<GetSwapInResponse>(
            self.0.code_hash.clone(),
            self.0.address.clone(),
            &QueryMsg::GetSwapIn {
                lb_pair,
                amount_out,
                swap_for_y,
            },
        )
    }

    pub fn get_swap_out(
        &self,
        querier: QuerierWrapper,
        lb_pair: ContractInfo,
        amount_in: Uint128,
        swap_for_y: bool,
    ) -> StdResult<GetSwapOutResponse> {
        querier.query_wasm_smart::<GetSwapOutResponse>(
            self.0.code_hash.clone(),
            self.0.address.clone(),
            &QueryMsg::GetSwapOut {
                lb_pair,
                amount_in,
                swap_for_y,
            },
        )
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub factory: ContractInfo,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateLbPair {
        token_x: TokenType,
        token_y: TokenType,
        active_id: u32,
        bin_step: u16,
    },
    AddLiquidity {
        liquidity_parameters: LiquidityParameters,
    },
    AddLiquidityNative {
        liquidity_parameters: LiquidityParameters,
    },
    RemoveLiquidity {
        token_x: ContractInfo,
        token_y: ContractInfo,
        bin_step: u16,
        amount_x_min: Uint128,
        amount_y_min: Uint128,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
        to: String,
        deadline: Uint64,
    },
    RemoveLiquidityNative {
        token: TokenType,
        bin_step: u16,
        amount_token_min: Uint128,
        amount_native_min: Uint128,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
        to: String,
        deadline: Uint64,
    },
    SwapExactTokensForTokens {
        amount_in: Uint128,
        amount_out_min: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    SwapExactTokensForNative {
        amount_in: Uint128,
        amount_out_min_native: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    SwapExactNativeforTokens {
        amount_out_min: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    SwapTokensForExactTokens {
        amount_out: Uint128,
        amount_in_max: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    SwapTokensForExactNative {
        amount_native_out: Uint128,
        amount_in_max: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    SwapNativeforExactTokens {
        amount_out: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    SwapExactTokensForTokensSupportingFeeOnTransferTokens {
        amount_in: Uint128,
        amount_out_min: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    SwapExactTokensForNativesupportingFeeOnTransferTokens {
        amount_in: Uint128,
        amount_out_min_native: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    SwapExactNativeforTokensSupportingFeeOnTransferTokens {
        amount_out_min: Uint128,
        path: Path,
        to: String,
        deadline: Uint64,
    },
    Sweep {
        token: TokenType,
        to: String,
        amount: Uint128,
    },
    SweepLbToken {
        token: ContractInfo, // must be an LbToken
        to: String,
        ids: Vec<u32>,
        amounts: Vec<Uint128>,
    },

    // not in joe-v2
    Register {
        address: String,
        code_hash: String,
    },
    RegisterBatch {
        tokens: Vec<RawContract>,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<String>,
        msg: Binary,
    },
}

#[cw_serde]
pub struct CreateLbPairResponse {
    pub lb_pair: LbPair,
}

#[cw_serde]
pub struct AddLiquidityResponse {
    pub amount_x_added: Uint128,
    pub amount_y_added: Uint128,
    pub amount_x_left: Uint128,
    pub amount_y_left: Uint128,
    pub deposit_ids: Vec<u32>,
    pub liquidity_minted: Vec<Uint256>,
}

#[cw_serde]
pub struct RemoveLiquidityResponse {
    pub amount_x: Uint128,
    pub amount_y: Uint128,
}

#[cw_serde]
pub struct SwapResponse {
    pub amount_out: Uint128,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetFactoryResponse)]
    GetFactory {},

    #[returns(GetIdFromPriceResponse)]
    GetIdFromPrice {
        lb_pair: ContractInfo,
        price: Uint256,
    },

    #[returns(GetPriceFromIdResponse)]
    GetPriceFromId { lb_pair: ContractInfo, id: u32 },

    #[returns(GetSwapInResponse)]
    GetSwapIn {
        lb_pair: ContractInfo,
        amount_out: Uint128,
        swap_for_y: bool,
    },

    #[returns(GetSwapOutResponse)]
    GetSwapOut {
        lb_pair: ContractInfo,
        amount_in: Uint128,
        swap_for_y: bool,
    },
}

#[cw_serde]
pub struct GetFactoryResponse {
    pub factory: Addr,
}

#[cw_serde]
pub struct GetIdFromPriceResponse {
    pub id: u32,
}

#[cw_serde]
pub struct GetPriceFromIdResponse {
    pub price: Uint256,
}

#[cw_serde]
pub struct GetSwapInResponse {
    pub amount_in: Uint128,
    pub amount_out_left: Uint128,
    pub fee: Uint128,
}

#[cw_serde]
pub struct GetSwapOutResponse {
    pub amount_in_left: Uint128,
    pub amount_out: Uint128,
    pub fee: Uint128,
}
