use crate::lb_pair::{LbPair, LiquidityParameters};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    Addr, Binary, ContractInfo, QuerierWrapper, StdResult, Uint128, Uint256, Uint64,
};
use shade_protocol::contract_interfaces::swap::core::TokenType;

pub struct ILbRouter(pub ContractInfo);

impl ILbRouter {
    pub fn get_swap_out(
        &self,
        querier: QuerierWrapper,
        lb_pair: ContractInfo,
        amount_in: Uint128,
        swap_for_y: bool,
    ) -> StdResult<SwapOutResponse> {
        querier.query_wasm_smart::<SwapOutResponse>(
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
        token: ContractInfo, // must be a snip20 token
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
    // TODO: make this a vec of ContractInfo
    Register {
        address: String,
        code_hash: String,
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

/// The path parameters, such as:
/// - pair_bin_steps: The list of bin steps of the pairs to go through
/// - versions: The list of versions of the pairs to go through
/// - token_path: The list of tokens in the path to go through
#[cw_serde]
pub struct Path {
    pub pair_bin_steps: Vec<u16>,
    pub versions: Vec<Version>,
    // TODO: Does this need to be Vec<RawContract> instead?
    pub token_path: Vec<TokenType>, // contracts that implements the snip20 interface
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FactoryResponse)]
    GetFactory {},

    #[returns(IdFromPriceResponse)]
    GetIdFromPrice {
        lb_pair: ContractInfo,
        price: Uint256,
    },

    #[returns(PriceFromIdResponse)]
    GetPriceFromId { lb_pair: ContractInfo, id: u32 },

    #[returns(SwapInResponse)]
    GetSwapIn {
        lb_pair: ContractInfo,
        amount_out: Uint128,
        swap_for_y: bool,
    },

    #[returns(SwapOutResponse)]
    GetSwapOut {
        lb_pair: ContractInfo,
        amount_in: Uint128,
        swap_for_y: bool,
    },
}

#[cw_serde]
pub struct FactoryResponse {
    pub factory: Addr,
}

#[cw_serde]
pub struct IdFromPriceResponse {
    pub id: u32,
}

#[cw_serde]
pub struct PriceFromIdResponse {
    pub price: Uint256,
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
