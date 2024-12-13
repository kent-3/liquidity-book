use super::lb_factory::{ContractImplementation, StaticFeeParameters};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    Addr, Binary, ContractInfo, QuerierWrapper, StdResult, Uint128, Uint256, Uint64,
};
use lb_libraries::hooks::Parameters;
use lb_libraries::types::{Bytes32, LiquidityConfiguration};
use shade_protocol::{
    swap::core::{TokenAmount, TokenType},
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, Query},
};
use std::fmt::{Debug, Display};

pub struct ILbPair(pub ContractInfo);

impl ILbPair {
    pub fn get_token_x(&self, querier: QuerierWrapper) -> StdResult<ContractInfo> {
        querier
            .query_wasm_smart::<TokenXResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetTokenX {},
            )
            .map(|response| response.token_x.into_contract_info().unwrap())
    }

    pub fn get_token_y(&self, querier: QuerierWrapper) -> StdResult<ContractInfo> {
        querier
            .query_wasm_smart::<TokenYResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetTokenY {},
            )
            .map(|response| response.token_y.into_contract_info().unwrap())
    }

    pub fn get_active_id(&self, querier: QuerierWrapper) -> StdResult<u32> {
        querier
            .query_wasm_smart::<ActiveIdResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetTokenY {},
            )
            .map(|response| response.active_id)
    }
}

// added this directly to avoid using the "snip20" feature of shade-protocol, which brings in
// secret-storage-plus as a dependency, which was causing issues.
#[cw_serde]
pub struct Snip20ReceiveMsg {
    pub sender: String,
    pub from: String,
    pub amount: Uint128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub msg: Option<Binary>,
}

#[cw_serde]
pub struct LbPair {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub contract: ContractInfo,
}

#[cw_serde]
pub struct LbPairInformation {
    pub bin_step: u16,
    pub lb_pair: LbPair,
    pub created_by_owner: bool,
    pub ignored_for_routing: bool,
}

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

#[cw_serde]
pub struct RemoveLiquidity {
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub amount_x_min: Uint128,
    pub amount_y_min: Uint128,
    pub ids: Vec<u32>,
    pub amounts: Vec<Uint256>,
    pub deadline: Uint64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub factory: ContractInfo,
    pub token_x: TokenType,
    pub token_y: TokenType,
    pub bin_step: u16,
    pub pair_parameters: StaticFeeParameters,
    pub active_id: u32,
    pub lb_token_implementation: ContractImplementation,
    pub viewing_key: String,
    pub entropy: String,
    pub protocol_fee_recipient: Addr,
    // TODO: Decide about getting rid of these.
    pub admin_auth: RawContract,
    pub query_auth: RawContract,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        offer: TokenAmount,
        expected_return: Option<Uint128>,
        to: Option<String>,
        padding: Option<String>,
    },
    FlashLoan {},
    Mint {
        to: String,
        // TODO: Change to the new encoded Bytes32 approach.
        liquidity_configs: Vec<LiquidityConfiguration>,
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
    ForceDecay {},

    // not in joe-v2
    SetContractStatus {
        contract_status: ContractStatus,
    },
    Receive(Snip20ReceiveMsg),
}

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
    SwapTokens {
        expected_return: Option<Uint128>,
        to: Option<String>,
        padding: Option<String>,
    },
}

impl ExecuteCallback for InvokeMsg {
    const BLOCK_SIZE: usize = 256;
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

    // not in joe-v2
    #[returns(LbTokenResponse)]
    GetLbToken {},
    #[returns(LbTokenSupplyResponse)]
    GetLbTokenSupply { id: u32 },
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

#[cw_serde]
pub struct LbHooksParametersResponse {
    pub hooks_parameters: Parameters,
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
    pub contract: ContractInfo,
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
