use super::lb_pair::{LbPair, LbPairInformation};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, ContractInfo, QuerierWrapper, StdResult};
use shade_protocol::{
    swap::core::TokenType,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, Query},
};

pub struct ILbFactory(pub ContractInfo);

impl ILbFactory {
    pub fn get_all_lb_pairs(
        &self,
        querier: QuerierWrapper,
        token_x: &ContractInfo,
        token_y: &ContractInfo,
    ) -> StdResult<Vec<LbPairInformation>> {
        let token_x: TokenType = token_x.clone().into();
        let token_y: TokenType = token_y.clone().into();

        // which style is better?

        // A:

        // let AllLbPairsResponse { lb_pairs_available } = querier.query_wasm_smart(
        //     self.0.code_hash.clone(),
        //     self.0.address.clone(),
        //     &QueryMsg::GetAllLbPairs {
        //         token_x: token_x.into(),
        //         token_y: token_y.into(),
        //     },
        // )?;
        //
        // Ok(lb_pairs_available)

        // B:

        querier
            .query_wasm_smart::<AllLbPairsResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetAllLbPairs { token_x, token_y },
            )
            .map(|response| response.lb_pairs_available)
    }

    pub fn get_lb_pair_information(
        &self,
        querier: QuerierWrapper,
        token_x: ContractInfo,
        token_y: ContractInfo,
        bin_step: u16,
    ) -> StdResult<LbPairInformation> {
        // let LbPairInformationResponse {
        //     lb_pair_information,
        // } = querier.query_wasm_smart(
        //     self.0.code_hash.clone(),
        //     self.0.address.clone(),
        //     &QueryMsg::GetLbPairInformation {
        //         token_x: token_x.into(),
        //         token_y: token_y.into(),
        //         bin_step,
        //     },
        // )?;
        //
        // Ok(lb_pair_information)

        querier
            .query_wasm_smart::<LbPairInformationResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetLbPairInformation {
                    token_x: token_x.into(),
                    token_y: token_y.into(),
                    bin_step,
                },
            )
            .map(|response| response.lb_pair_information)
    }
}

// pub trait ILbFactory {
//     fn get_all_lb_pairs(
//         &self,
//         querier: QuerierWrapper,
//         token_x: ContractInfo,
//         token_y: ContractInfo,
//     ) -> StdResult<Vec<LbPairInformation>>;
//     fn get_lb_pair_information(
//         &self,
//         querier: QuerierWrapper,
//         token_x: ContractInfo,
//         token_y: ContractInfo,
//         bin_step: u16,
//     ) -> StdResult<LbPairInformation>;
// }
//
// impl ILbFactory for ContractInfo {
//     fn get_all_lb_pairs(
//         &self,
//         querier: QuerierWrapper,
//         token_x: ContractInfo,
//         token_y: ContractInfo,
//     ) -> StdResult<Vec<LbPairInformation>> {
//         // which style is better?
//
//         // A:
//
//         // let AllLbPairsResponse { lb_pairs_available } = querier.query_wasm_smart(
//         //     self.code_hash.clone(),
//         //     self.address.clone(),
//         //     &QueryMsg::GetAllLbPairs {
//         //         token_x: token_x.into(),
//         //         token_y: token_y.into(),
//         //     },
//         // )?;
//         //
//         // Ok(lb_pairs_available)
//
//         // B:
//
//         querier
//             .query_wasm_smart::<AllLbPairsResponse>(
//                 self.code_hash.clone(),
//                 self.address.clone(),
//                 &QueryMsg::GetAllLbPairs {
//                     token_x: token_x.into(),
//                     token_y: token_y.into(),
//                 },
//             )
//             .map(|response| response.lb_pairs_available)
//     }
//
//     fn get_lb_pair_information(
//         &self,
//         querier: QuerierWrapper,
//         token_x: ContractInfo,
//         token_y: ContractInfo,
//         bin_step: u16,
//     ) -> StdResult<LbPairInformation> {
//         // let LbPairInformationResponse {
//         //     lb_pair_information,
//         // } = querier.query_wasm_smart(
//         //     self.code_hash.clone(),
//         //     self.address.clone(),
//         //     &QueryMsg::GetLbPairInformation {
//         //         token_x: token_x.into(),
//         //         token_y: token_y.into(),
//         //         bin_step,
//         //     },
//         // )?;
//         //
//         // Ok(lb_pair_information)
//
//         querier
//             .query_wasm_smart::<LbPairInformationResponse>(
//                 self.code_hash.clone(),
//                 self.address.clone(),
//                 &QueryMsg::GetLbPairInformation {
//                     token_x: token_x.into(),
//                     token_y: token_y.into(),
//                     bin_step,
//                 },
//             )
//             .map(|response| response.lb_pair_information)
//     }
// }

#[cw_serde]
#[derive(Default)]
pub struct ContractImplementation {
    pub id: u64,
    pub code_hash: String,
}

#[cw_serde]
pub struct StaticFeeParameters {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub variable_fee_control: u32,
    pub protocol_share: u16,
    pub max_volatility_accumulator: u32,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub query_auth: RawContract,
    pub owner: Option<Addr>,
    pub fee_recipient: Addr,
}
impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    SetLbPairImplementation {
        implementation: ContractImplementation,
    },
    SetLbTokenImplementation {
        implementation: ContractImplementation,
    },
    CreateLbPair {
        token_x: TokenType,
        token_y: TokenType,
        // u24
        active_id: u32,
        bin_step: u16,
        viewing_key: String,
        entropy: String,
    },
    SetLbPairIgnored {
        token_x: TokenType,
        token_y: TokenType,
        bin_step: u16,
        ignored: bool,
    },
    SetPairPreset {
        bin_step: u16,
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        // u24
        variable_fee_control: u32,
        protocol_share: u16,
        // u24
        max_volatility_accumulator: u32,
        is_open: bool,
    },
    SetPresetOpenState {
        bin_step: u16,
        is_open: bool,
    },
    RemovePreset {
        bin_step: u16,
    },
    SetFeeParametersOnPair {
        token_x: TokenType,
        token_y: TokenType,
        bin_step: u16,
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        // u24
        variable_fee_control: u32,
        protocol_share: u16,
        // u24
        max_volatility_accumulator: u32,
    },
    SetFeeRecipient {
        fee_recipient: Addr,
    },

    AddQuoteAsset {
        asset: TokenType,
    },
    RemoveQuoteAsset {
        asset: TokenType,
    },
    ForceDecay {
        pair: LbPair,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MinBinStepResponse)]
    GetMinBinStep {},
    #[returns(FeeRecipientResponse)]
    GetFeeRecipient {},
    // #[returns(MaxFlashLoanFeeResponse)]
    // GetMaxFlashLoanFee {},
    // #[returns(FlashLoanFeeResponse)]
    // GetFlashLoanFee {},
    #[returns(LbPairImplementationResponse)]
    GetLbPairImplementation {},
    #[returns(LbTokenImplementationResponse)]
    GetLbTokenImplementation {},
    #[returns(NumberOfLbPairsResponse)]
    GetNumberOfLbPairs {},
    #[returns(LbPairAtIndexResponse)]
    GetLbPairAtIndex { index: u32 },
    #[returns(NumberOfQuoteAssetsResponse)]
    GetNumberOfQuoteAssets {},
    #[returns(QuoteAssetAtIndexResponse)]
    GetQuoteAssetAtIndex { index: u32 },
    #[returns(IsQuoteAssetResponse)]
    IsQuoteAsset { token: TokenType },
    #[returns(LbPairInformationResponse)]
    GetLbPairInformation {
        token_x: TokenType,
        token_y: TokenType,
        bin_step: u16,
    },
    #[returns(PresetResponse)]
    GetPreset { bin_step: u16 },
    #[returns(AllBinStepsResponse)]
    GetAllBinSteps {},
    #[returns(OpenBinStepsResponse)]
    GetOpenBinSteps {},
    #[returns(AllLbPairsResponse)]
    GetAllLbPairs {
        token_x: TokenType,
        token_y: TokenType,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

// We define a custom struct for each query response
#[cw_serde]
pub struct MinBinStepResponse {
    pub min_bin_step: u8,
}

#[cw_serde]
pub struct FeeRecipientResponse {
    pub fee_recipient: Addr,
}

#[cw_serde]
pub struct LbPairImplementationResponse {
    pub lb_pair_implementation: ContractImplementation,
}

#[cw_serde]
pub struct LbTokenImplementationResponse {
    pub lb_token_implementation: ContractImplementation,
}

#[cw_serde]
pub struct NumberOfLbPairsResponse {
    pub lb_pair_number: u32,
}

#[cw_serde]
pub struct LbPairAtIndexResponse {
    pub lb_pair: LbPair,
}

#[cw_serde]
pub struct NumberOfQuoteAssetsResponse {
    pub number_of_quote_assets: u32,
}

#[cw_serde]
pub struct QuoteAssetAtIndexResponse {
    pub asset: TokenType,
}

#[cw_serde]
pub struct IsQuoteAssetResponse {
    pub is_quote: bool,
}

#[cw_serde]
pub struct LbPairInformationResponse {
    pub lb_pair_information: LbPairInformation,
}

#[cw_serde]
pub struct PresetResponse {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    // u24
    pub variable_fee_control: u32,
    pub protocol_share: u16,
    // u24
    pub max_volatility_accumulator: u32,
    pub is_open: bool,
}

#[cw_serde]
pub struct AllBinStepsResponse {
    pub bin_step_with_preset: Vec<u16>,
}

#[cw_serde]
pub struct OpenBinStepsResponse {
    pub open_bin_steps: Vec<u16>,
}

#[cw_serde]
pub struct AllLbPairsResponse {
    pub lb_pairs_available: Vec<LbPairInformation>,
}
