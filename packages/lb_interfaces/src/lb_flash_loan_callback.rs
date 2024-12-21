//! # Liquidity Book Flash Loan Callback Interface
//! Required interface to interact with LB flash loans

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, ContractInfo};
use lb_libraries::Bytes32;

pub trait ILbFlashLoanCallback {
    fn lb_flash_loan_callback(
        address: String,
        token_x: ContractInfo, // snip20
        token_y: ContractInfo, // snip20
        amounts: Bytes32,
        total_fees: Bytes32,
        data: Option<Binary>,
    ) -> Bytes32;
}

#[cw_serde]
pub enum ExecuteMsg {
    LbFlashLoanCallback {
        address: String,
        token_x: ContractInfo, // snip20
        token_y: ContractInfo, // snip20
        amounts: Bytes32,
        total_fees: Bytes32,
        data: Option<Binary>,
    },
}

// TODO: Instructions unclear.
