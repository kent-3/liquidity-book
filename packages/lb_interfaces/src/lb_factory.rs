use super::lb_pair::{LbPair, LbPairInformation};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, ContractInfo, Event, QuerierWrapper, StdResult, Uint128};
use serde::{Deserialize, Serialize};
use shade_protocol::{
    swap::core::TokenType,
    utils::{asset::RawContract, ExecuteCallback, InstantiateCallback, Query},
};
use std::ops::Deref;

pub trait LbFactoryEventExt {
    fn lb_pair_created(
        token_x: String,
        token_y: String,
        bin_step: u16,
        lb_pair: String,
        pid: u32,
    ) -> Event {
        Event::new("lb_pair_created")
            .add_attribute_plaintext("token_x", token_x)
            .add_attribute_plaintext("token_y", token_y)
            .add_attribute_plaintext("bin_step", bin_step.to_string())
            .add_attribute_plaintext("lb_pair", lb_pair)
            .add_attribute_plaintext("pid", pid.to_string())
    }

    fn fee_recipient_set(old_recipient: Addr, new_recipient: Addr) -> Event {
        Event::new("fee_recipient_set")
            .add_attribute_plaintext("old_recipient", old_recipient)
            .add_attribute_plaintext("new_recipient", new_recipient)
    }

    fn flash_loan_fee_set(old_flash_loan_fee: Uint128, new_flash_loan_fee: Uint128) -> Event {
        Event::new("flash_loan_fee_set")
            .add_attribute_plaintext("old_flash_loan_fee", old_flash_loan_fee)
            .add_attribute_plaintext("new_flash_loan_fee", new_flash_loan_fee)
    }

    fn lb_pair_implementation_set(
        old_lb_pair_implementation: u64,
        new_lb_pair_implementation: u64,
    ) -> Event {
        Event::new("lb_pair_implementation_set")
            .add_attribute_plaintext(
                "old_lb_pair_implementation",
                old_lb_pair_implementation.to_string(),
            )
            .add_attribute_plaintext(
                "new_lb_pair_implementation",
                new_lb_pair_implementation.to_string(),
            )
    }

    fn lb_token_implementation_set(
        old_lb_token_implementation: u64,
        new_lb_token_implementation: u64,
    ) -> Event {
        Event::new("lb_token_implementation_set")
            .add_attribute_plaintext(
                "old_lb_token_implementation",
                old_lb_token_implementation.to_string(),
            )
            .add_attribute_plaintext(
                "new_lb_token_implementation",
                new_lb_token_implementation.to_string(),
            )
    }

    fn lb_pair_ignored_state_changed(lb_pair: String, ignored: bool) -> Event {
        Event::new("lb_pair_ignored_state_changed")
            .add_attribute_plaintext("lb_pair", lb_pair)
            .add_attribute_plaintext("ignored", ignored.to_string())
    }

    fn preset_set(
        bin_step: u16,
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        variable_fee_control: u32,
        protocol_share: u16,
        max_volatility_accumulator: u32,
    ) -> Event {
        Event::new("preset_set")
            .add_attribute_plaintext("bin_step", bin_step.to_string())
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

    fn preset_open_state_changed(bin_step: u16, is_open: bool) -> Event {
        Event::new("preset_open_state_changed")
            .add_attribute_plaintext("bin_step", bin_step.to_string())
            .add_attribute_plaintext("is_open", is_open.to_string())
    }

    fn preset_removed(bin_step: u16) -> Event {
        Event::new("preset_removed").add_attribute_plaintext("bin_step", bin_step.to_string())
    }

    fn quote_asset_added(quote_asset: String) -> Event {
        Event::new("quote_asset_added").add_attribute_plaintext("quote_asset", quote_asset)
    }

    fn quote_asset_removed(quote_asset: String) -> Event {
        Event::new("quote_asset_removed").add_attribute_plaintext("quote_asset", quote_asset)
    }
}

impl LbFactoryEventExt for Event {}

#[derive(Serialize, Deserialize)]
pub struct ILbFactory(pub ContractInfo);

impl Deref for ILbFactory {
    type Target = ContractInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ILbFactory {
    pub fn get_fee_recipient(&self, querier: QuerierWrapper) -> StdResult<Addr> {
        querier
            .query_wasm_smart::<FeeRecipientResponse>(
                self.0.code_hash.clone(),
                self.0.address.clone(),
                &QueryMsg::GetFeeRecipient {},
            )
            .map(|response| response.fee_recipient)
    }

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
    SetPreset {
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
    // TODO: Hooks
    SetLBHooksParametersOnPair,
    RemoveLBHooksOnPair,
    SetFeeRecipient {
        fee_recipient: Addr,
    },
    SetFlashLoanFee,
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
    #[returns(MaxFlashLoanFeeResponse)]
    GetMaxFlashLoanFee {},
    #[returns(FlashLoanFeeResponse)]
    GetFlashLoanFee {},
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

#[cw_serde]
pub struct MinBinStepResponse {
    pub min_bin_step: u8,
}

#[cw_serde]
pub struct FeeRecipientResponse {
    pub fee_recipient: Addr,
}

#[cw_serde]
pub struct MaxFlashLoanFeeResponse {
    pub max_flash_loan_fee: Uint128,
}

#[cw_serde]
pub struct FlashLoanFeeResponse {
    pub flash_loan_fee: Uint128,
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
