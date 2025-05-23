use super::lb_factory::{Implementation, StaticFeeParameters};
use crate::core::{
    callback::{ExecuteCallback, InstantiateCallback, Query},
    RawContract, TokenType,
};
use crate::libraries::{hooks::HooksParameters, Bytes32, LiquidityConfigurations};
use base64::prelude::{Engine as _, BASE64_STANDARD};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    to_binary, Addr, Binary, ContractInfo, Event, QuerierWrapper, StdResult, Uint128, Uint256,
    WasmMsg,
};
use std::fmt::{Debug, Display};
use std::ops::Deref;

use crate::libraries::{
    bin_helper::BinError,
    fee_helper::FeeError,
    math::{
        liquidity_configurations::LiquidityConfigurationsError,
        packed_u128_math::PackedUint128MathError, u128x128_math::U128x128MathError,
        u256x256_math::U256x256MathError,
    },
    oracle_helper::OracleError,
    pair_parameter_helper::PairParametersError,
};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

// TODO: reorder these to match the original

#[derive(thiserror::Error, Debug)]
pub enum LbPairError {
    #[error("Zero borrow amount!")]
    ZeroBorrowAmount,
    #[error("Address is zero!")]
    AddressZero,
    #[error("Empty liquidity configurations!")]
    EmptyMarketConfigs,
    #[error("Flash loan callback failed!")]
    FlashLoanCallbackFailed,
    #[error("Flash loan insufficient amount!")]
    FlashLoanInsufficientAmount,
    #[error("Insufficient amount in!")]
    InsufficientAmountIn,
    #[error("Insufficient amount out!")]
    InsufficientAmountOut,
    #[error("Invalid input!")]
    InvalidInput,
    #[error("Invalid static fee parameters!")]
    InvalidStaticFeeParameters,
    #[error("Only the Factory can do that!")]
    OnlyFactory,
    #[error("Only the Protocol Fee Recipient can do that!")]
    OnlyProtocolFeeRecipient,
    #[error("Not enough liquidity!")]
    OutOfLiquidity,
    #[error("Token not supported!")]
    TokenNotSupported(),
    #[error("Zero amount for bin {id}!")]
    ZeroAmount { id: u32 },
    #[error(
        "Zero amounts out for bin {id}! amount to burn: {amount_to_burn} total supply: {total_supply}"
    )]
    ZeroAmountsOut {
        id: u32,
        amount_to_burn: Uint256,
        total_supply: Uint256,
    },
    #[error("Zero Shares for bin {id}!")]
    ZeroShares { id: u32 },
    #[error("Max total fee exceeded!")]
    MaxTotalFeeExceeded,
    #[error("Invalid hooks!")]
    InvalidHooks,

    // --- not in joe-v2 ---
    //
    #[error("{0}!")]
    Generic(String),
    #[error("Value greater than u24!")]
    U24Overflow,
    #[error("Transaction is blocked by contract status!")]
    TransactionBlock(),
    #[error("Receiver callback \"msg\" parameter cannot be empty!")]
    ReceiverMsgEmpty,
    #[error("Unknown reply id: {id}!")]
    UnknownReplyId { id: u64 },
    #[error("Reply data is missing!")]
    ReplyDataMissing,

    // Error Wrappings from Dependencies
    #[error(transparent)]
    StdError(#[from] cosmwasm_std::StdError),
    #[error(transparent)]
    ConversionOverflowError(#[from] cosmwasm_std::ConversionOverflowError),

    #[error(transparent)]
    LbTokenError(#[from] super::lb_token2::LbTokenError),
    #[error(transparent)]
    LbErr(#[from] crate::libraries::Error),
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
    #[error(transparent)]
    PackedUint128MathError(#[from] PackedUint128MathError),

    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
}

#[cw_serde]
pub struct LbPair {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub contract: ContractInfo,
}

#[cw_serde]
pub struct Snip20ReceiveMsg {
    pub sender: String,
    pub from: String,
    pub amount: Uint128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub msg: Option<Binary>,
}

// TODO: Decide which attributes to make private.
// NOTE: All Bytes32 values are represented as Base64 strings. Should we use hex instead?
// TODO: We are doing a lot of unwrapping here. Is that ok?
pub trait LbPairEventExt {
    fn deposited_to_bins(sender: &Addr, to: &Addr, ids: &[u32], amounts: &[Bytes32]) -> Event {
        let amounts: Vec<String> = amounts
            .iter()
            .map(|amount| BASE64_STANDARD.encode(amount))
            .collect();

        Event::new("deposited_to_bins")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("to", to)
            .add_attribute_plaintext("ids", serde_json_wasm::to_string(&ids).unwrap())
            .add_attribute_plaintext("amounts", serde_json_wasm::to_string(&amounts).unwrap())
    }

    fn withdrawn_from_bins(sender: &Addr, to: &Addr, ids: &[u32], amounts: &[Bytes32]) -> Event {
        let amounts: Vec<String> = amounts
            .iter()
            .map(|amount| BASE64_STANDARD.encode(amount))
            .collect();

        Event::new("withdrawn_from_bins")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("to", to)
            .add_attribute_plaintext("ids", serde_json_wasm::to_string(&ids).unwrap())
            .add_attribute_plaintext("amounts", serde_json_wasm::to_string(&amounts).unwrap())
    }

    fn composition_fees(
        sender: &Addr,
        id: u32,
        total_fees: &Bytes32,
        protocol_fees: &Bytes32,
    ) -> Event {
        Event::new("composition_fees")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("id", id.to_string())
            .add_attribute_plaintext("total_fees", BASE64_STANDARD.encode(total_fees))
            .add_attribute_plaintext("protocol_fees", BASE64_STANDARD.encode(protocol_fees))
    }

    fn collected_protocol_fees(fee_recipient: &Addr, protocol_fees: &Bytes32) -> Event {
        Event::new("collected_protocol_fees")
            .add_attribute_plaintext("fee_recipient", fee_recipient)
            .add_attribute_plaintext("protocol_fees", BASE64_STANDARD.encode(protocol_fees))
    }

    fn swap(
        sender: &str,
        to: &str,
        id: u32,
        amounts_in: Bytes32,
        amounts_out: Bytes32,
        volatility_accumulator: u32,
        total_fees: Bytes32,
        protocol_fees: Bytes32,
    ) -> Event {
        Event::new("swap")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("to", to)
            .add_attribute_plaintext("id", id.to_string())
            .add_attribute_plaintext("amounts_in", BASE64_STANDARD.encode(amounts_in))
            .add_attribute_plaintext("amounts_out", BASE64_STANDARD.encode(amounts_out))
            .add_attribute_plaintext("volatility_accumulator", volatility_accumulator.to_string())
            .add_attribute_plaintext("total_fees", BASE64_STANDARD.encode(total_fees))
            .add_attribute_plaintext("protocol_fees", BASE64_STANDARD.encode(protocol_fees))
    }

    fn static_fee_parameters_set(
        sender: &Addr,
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        variable_fee_control: u32,
        protocol_share: u16,
        max_volatility_accumulator: u32,
    ) -> Event {
        Event::new("static_fee_parameters_set")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("base_factor", base_factor.to_string())
            .add_attribute_plaintext("filter_period", filter_period.to_string())
            .add_attribute_plaintext("decay_period", decay_period.to_string())
            .add_attribute_plaintext("reduction_factor", reduction_factor.to_string())
            .add_attribute_plaintext("variable_fee_control", variable_fee_control.to_string())
            .add_attribute_plaintext("protocol_share", protocol_share.to_string())
            .add_attribute_plaintext(
                "max_volatility_accumulator",
                max_volatility_accumulator.to_string(),
            )
    }

    fn hooks_parameters_set(sender: &Addr, hooks_parameters: &Option<HooksParameters>) -> Event {
        Event::new("hooks_parameters_set")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext(
                "hooks_parameters",
                serde_json_wasm::to_string(&hooks_parameters).unwrap(),
            )
    }

    fn flash_loan(
        sender: &Addr,
        receiver: &Addr,
        active_id: u32,
        amounts: &Bytes32,
        total_fees: &Bytes32,
        protocol_fees: &Bytes32,
    ) -> Event {
        Event::new("flash_loan")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("receiver", receiver)
            .add_attribute_plaintext("active_id", active_id.to_string())
            .add_attribute_plaintext("amounts", BASE64_STANDARD.encode(amounts))
            .add_attribute_plaintext("total_fees", BASE64_STANDARD.encode(total_fees))
            .add_attribute_plaintext("protocol_fees", BASE64_STANDARD.encode(protocol_fees))
    }

    fn oracle_length_increased(sender: &Addr, oracle_length: u16) -> Event {
        Event::new("oracle_length_increased")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("oracle_length", oracle_length.to_string())
    }

    fn forced_decay(sender: &Addr, id_reference: u32, volatility_reference: u32) -> Event {
        Event::new("forced_decay")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("id_reference", id_reference.to_string())
            .add_attribute_plaintext("volatility_reference", volatility_reference.to_string())
    }
}

impl LbPairEventExt for Event {}

#[cw_serde]
pub struct InstantiateMsg {
    pub factory: ContractInfo,
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub pair_parameters: StaticFeeParameters,
    pub active_id: u32,
    pub lb_token_implementation: Implementation,
    pub viewing_key: String,
    pub entropy: String,
    // TODO: Decide about getting rid of these.
    pub admin_auth: RawContract,
    pub query_auth: RawContract,
}

// impl InstantiateCallback for InstantiateMsg {
//     const BLOCK_SIZE: usize = 256;
// }

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        swap_for_y: bool,
        to: String,
    },
    // TODO: figure out proper types here
    FlashLoan {
        receiver: ContractInfo,
        amounts: Bytes32,
        data: Option<Binary>,
    },
    Mint {
        to: String,
        liquidity_configs: Vec<LiquidityConfigurations>,
        refund_to: String,
    },
    Burn {
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts_to_burn: Vec<Uint256>,
    },
    CollectProtocolFees {},
    IncreaseOracleLength {
        new_length: u16,
    },
    SetStaticFeeParameters {
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        variable_fee_control: u32,
        protocol_share: u16,
        max_volatility_accumulator: u32,
    },
    SetHooksParameters {
        hooks_parameters: Option<HooksParameters>,
        on_hooks_set_data: Option<Binary>,
    },
    ForceDecay {},
    BatchTransferFrom {
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    },

    // lb-token
    ApproveForAll {
        spender: String,
        approved: bool,
    },
    // BatchTransferFrom {
    //     from: String,
    //     to: String,
    //     ids: Vec<u32>,
    //     amounts: Vec<Uint256>,
    // },

    // not in joe-v2
    SetContractStatus {
        contract_status: ContractStatus,
    },
    // Receive(Snip20ReceiveMsg),
}

// TODO: should this be here at all?
#[cw_serde]
pub enum ContractStatus {
    Active,         // allows all operations
    FreezeAll,      // blocks everything except admin-protected config changes
    LpWithdrawOnly, // blocks everything except LP withdraws and admin-protected config changes
}

impl Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum InvokeMsg {
    // TODO: do we need a separate InvokeMsg for swaps?
    Swap { swap_for_y: bool, to: String },
}

// impl ExecuteCallback for InvokeMsg {
//     const BLOCK_SIZE: usize = 256;
// }

// TODO: this could instead return just the Bytes32, since it's mainly used by lb-router
#[cw_serde]
pub struct SwapResponse {
    pub amounts_out: Bytes32,
}

#[cw_serde]
pub struct MintResponse {
    pub amounts_received: Bytes32,
    pub amounts_left: Bytes32,
    pub liquidity_minted: Vec<Uint256>,
}

#[cw_serde]
pub struct BurnResponse {
    pub amounts: Vec<Bytes32>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FactoryResponse)]
    GetFactory {},
    #[returns(TokenXResponse)]
    GetTokenX {},
    #[returns(TokenYResponse)]
    GetTokenY {},
    #[returns(BinStepResponse)]
    GetBinStep {},
    #[returns(ReservesResponse)]
    GetReserves {},
    #[returns(ActiveIdResponse)]
    GetActiveId {},
    #[returns(BinResponse)]
    GetBin { id: u32 },
    #[returns(NextNonEmptyBinResponse)]
    GetNextNonEmptyBin { swap_for_y: bool, id: u32 },
    #[returns(ProtocolFeesResponse)]
    GetProtocolFees {},
    #[returns(StaticFeeParametersResponse)]
    GetStaticFeeParameters {},
    #[returns(LbHooksParametersResponse)]
    GetLbHooksParameters {},
    #[returns(VariableFeeParametersResponse)]
    GetVariableFeeParameters {},
    #[returns(OracleParametersResponse)]
    GetOracleParameters {},
    #[returns(OracleSampleAtResponse)]
    GetOracleSampleAt { lookup_timestamp: u64 },
    #[returns(PriceFromIdResponse)]
    GetPriceFromId { id: u32 },
    #[returns(IdFromPriceResponse)]
    GetIdFromPrice { price: Uint256 },
    #[returns(SwapInResponse)]
    GetSwapIn {
        amount_out: Uint128,
        swap_for_y: bool,
    },
    #[returns(SwapOutResponse)]
    GetSwapOut {
        amount_in: Uint128,
        swap_for_y: bool,
    },

    // lb-token
    #[returns(String)]
    Name {},
    #[returns(String)]
    Symbol {},
    #[returns(Uint256)]
    TotalSupply { id: u32 },
    #[returns(Uint256)]
    BalanceOf { account: String, id: u32 },
    #[returns(Vec<Uint256>)]
    BalanceOfBatch {
        accounts: Vec<String>,
        ids: Vec<u32>,
    },
    #[returns(bool)]
    IsApprovedForAll { owner: String, spender: String },

    // not in joe-v2
    #[returns(LbTokenResponse)]
    GetLbToken {}, // TODO: delete
    #[returns(LbTokenSupplyResponse)]
    GetLbTokenSupply { id: u32 }, // TODO: delete
    #[returns(BinsResponse)]
    GetBins { ids: Vec<u32> },
    #[returns(AllBinsResponse)]
    GetAllBins {
        id: Option<u32>,
        page: Option<u32>,
        page_size: Option<u32>,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

// TODO: should all the query response types start with "Get"?

#[cw_serde]
pub struct FactoryResponse {
    pub factory: Addr,
}

#[cw_serde]
pub struct TokenXResponse {
    pub token_x: TokenType,
}

#[cw_serde]
pub struct TokenYResponse {
    pub token_y: TokenType,
}

#[cw_serde]
pub struct BinStepResponse {
    pub bin_step: u16,
}

#[cw_serde]
pub struct ReservesResponse {
    pub reserve_x: Uint128,
    pub reserve_y: Uint128,
}

#[cw_serde]
pub struct ActiveIdResponse {
    pub active_id: u32,
}

#[cw_serde]
pub struct BinResponse {
    pub bin_id: u32,
    pub bin_reserve_x: Uint128,
    pub bin_reserve_y: Uint128,
}

#[cw_serde]
pub struct NextNonEmptyBinResponse {
    pub next_id: u32,
}

#[cw_serde]
pub struct ProtocolFeesResponse {
    pub protocol_fee_x: u128,
    pub protocol_fee_y: u128,
}

#[cw_serde]
pub struct StaticFeeParametersResponse {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub protocol_share: u16,
    pub max_volatility_accumulator: u32,
}

// TODO: decide on this. I think it's better to return flat data with generic types. but then I
// have to do annoying type conversions internally
#[cw_serde]
pub struct LbHooksParametersResponse {
    pub hooks_parameters: Option<HooksParameters>,
    // pub address: String,
    // pub code_hash: String,
    // pub flags: u16,
}

#[cw_serde]
pub struct VariableFeeParametersResponse {
    pub volatility_accumulator: u32,
    pub volatility_reference: u32,
    pub id_reference: u32,
    pub time_of_last_update: u64,
}

#[cw_serde]
#[derive(Default)]
pub struct OracleParametersResponse {
    pub sample_lifetime: u8,
    pub size: u16,
    pub active_size: u16,
    pub last_updated: u64,
    pub first_timestamp: u64,
}

// TODO: try to make this simpler. try returning a tuple (u64, u64, u64) instead.

#[cw_serde]
pub struct OracleSampleResponse {
    pub cumulative_id: u64,
    pub cumulative_volatility: u64,
    pub cumulative_bin_crossed: u64,
}

// TODO: Make a second oracle to track fee averages over time. To calculate APY.
#[cw_serde]
pub struct OracleSampleResponse2 {
    pub cumulative_id: u64,
    pub cumulative_provider_fee: u64,
    pub cumulative_protocol_fee: u64,
}

#[cw_serde]
pub struct OracleSampleAtResponse {
    pub sample: OracleSampleResponse,
}

#[cw_serde]
pub struct PriceFromIdResponse {
    pub price: Uint256,
}

#[cw_serde]
pub struct IdFromPriceResponse {
    pub id: u32,
}

#[cw_serde]
pub struct SwapInResponse {
    pub amount_in: Uint128,
    pub amount_out_left: Uint128,
    pub fee: Uint128,
}

#[cw_serde]
pub struct SwapOutResponse {
    pub amount_in_left: Uint128,
    pub amount_out: Uint128,
    pub fee: Uint128,
}

#[cw_serde]
pub struct LbTokenResponse {
    pub lb_token: ContractInfo,
}

#[cw_serde]
pub struct LbTokenSupplyResponse {
    pub total_supply: Uint256,
}

#[cw_serde]
pub struct BinsResponse(pub Vec<BinResponse>);

#[cw_serde]
pub struct AllBinsResponse {
    pub reserves: Vec<BinResponse>,
    pub last_id: u32,
    pub current_block_height: u64,
}

/// A thin wrapper around `ContractInfo` that provides additional
/// methods to interact with an LB Pair contract.
#[cw_serde]
pub struct ILbPair(pub ContractInfo);

impl Deref for ILbPair {
    type Target = ContractInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: think if there's a way to have the [LbPair] type also have these methods. that way it would
// have its associated tokens and bin step, in addition to the ContractInfo.

impl ILbPair {
    pub fn get_token_x(&self, querier: QuerierWrapper) -> StdResult<TokenType> {
        querier
            .query_wasm_smart::<TokenXResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetTokenX {},
            )
            .map(|response| response.token_x)
    }
    pub fn get_token_y(&self, querier: QuerierWrapper) -> StdResult<TokenType> {
        querier
            .query_wasm_smart::<TokenYResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetTokenY {},
            )
            .map(|response| response.token_y)
    }
    pub fn get_active_id(&self, querier: QuerierWrapper) -> StdResult<u32> {
        querier
            .query_wasm_smart::<ActiveIdResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetActiveId {},
            )
            .map(|response| response.active_id)
    }
    pub fn get_lb_hooks_parameters(
        &self,
        querier: QuerierWrapper,
    ) -> StdResult<Option<HooksParameters>> {
        querier
            .query_wasm_smart::<LbHooksParametersResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetLbHooksParameters {},
            )
            .map(|response| response.hooks_parameters)
    }

    pub fn swap(&self, swap_for_y: bool, to: String) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::Swap { swap_for_y, to };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn flash_loan(
        &self,
        receiver: ContractInfo,
        amounts: Bytes32,
        data: Option<Binary>,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::FlashLoan {
            receiver,
            amounts,
            data,
        };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn mint(
        &self,
        to: String,
        liquidity_configs: Vec<LiquidityConfigurations>,
        refund_to: String,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::Mint {
            to,
            liquidity_configs,
            refund_to,
        };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn burn(
        &self,
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts_to_burn: Vec<Uint256>,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::Burn {
            from,
            to,
            ids,
            amounts_to_burn,
        };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn collect_protocol_fees(&self) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::CollectProtocolFees {};

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn increase_oracle_length(&self, new_length: u16) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::IncreaseOracleLength { new_length };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn set_static_fee_parameters(
        &self,
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        variable_fee_control: u32,
        protocol_share: u16,
        max_volatility_accumulator: u32,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::SetStaticFeeParameters {
            base_factor,
            filter_period,
            decay_period,
            reduction_factor,
            variable_fee_control,
            protocol_share,
            max_volatility_accumulator,
        };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    // TODO: I need a way to set the hooks_parameters to None
    // should both arguments be Options?
    pub fn set_hooks_parameters(
        &self,
        hooks_parameters: Option<HooksParameters>,
        on_hooks_set_data: Option<Binary>,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::SetHooksParameters {
            hooks_parameters,
            on_hooks_set_data,
        };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn force_decay(&self) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::ForceDecay {};

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn batch_transfer_from(
        &self,
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    ) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::BatchTransferFrom {
            from,
            to,
            ids,
            amounts,
        };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
    pub fn set_contract_status(&self, contract_status: ContractStatus) -> StdResult<WasmMsg> {
        let msg = ExecuteMsg::SetContractStatus { contract_status };

        Ok(WasmMsg::Execute {
            contract_addr: self.address.to_string(),
            code_hash: self.code_hash.clone(),
            msg: to_binary(&msg)?,
            funds: vec![],
        })
    }
}
