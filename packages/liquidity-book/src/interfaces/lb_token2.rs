use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Event, Uint128, Uint256};

#[derive(thiserror::Error, Debug)]
pub enum LbTokenError {
    #[error("AddressThisOrZero")]
    AddressThisOrZero,
    #[error("InvalidLength")]
    InvalidLength,
    #[error("SelfApproval: {0}")]
    SelfApproval(String),
    #[error("SpenderNotApproved: from {from}, spender {spender}")]
    SpenderNotApproved { from: String, spender: String },
    #[error("TransferExceedsBalance: from {from}, id {id}, amount {amount}")]
    TransferExceedsBalance {
        from: Addr,
        id: u32,
        amount: Uint256,
    },
    #[error("BurnExceedsBalance: account {account}, id {id}, amount {amount}")]
    BurnExceedsBalance {
        account: Addr,
        id: u32,
        amount: Uint256,
    },

    #[error(transparent)]
    StdError(#[from] cosmwasm_std::StdError),
}

pub trait LbTokenEventExt {
    fn transfer_batch(
        sender: Addr,
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    ) -> Event {
        Event::new("transfer_batch")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("from", from)
            .add_attribute_plaintext("to", to)
            .add_attribute_plaintext("ids", format!("{:?}", ids))
            .add_attribute_plaintext("amounts", format!("{:?}", amounts))
    }

    fn approval_for_all(account: String, sender: String, approved: bool) -> Event {
        Event::new("approval_for_all")
            .add_attribute_plaintext("account", account)
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("approved", approved.to_string())
    }
}

impl LbTokenEventExt for Event {}

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    ApproveForAll {
        spender: String,
        approved: bool,
    },
    BatchTransferFrom {
        from: String,
        to: String,
        ids: Vec<u32>,
        amounts: Vec<Uint256>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(NameResponse)]
    Name,
    #[returns(SymbolResponse)]
    Symbol,
    #[returns(TotalSupplyResponse)]
    TotalSupply { id: u32 },
    #[returns(BalanceResponse)]
    BalanceOf { account: String, id: u32 },
    #[returns(BalanceBatchResponse)]
    BalanceOfBatch {
        accounts: Vec<String>,
        ids: Vec<u32>,
    },
    #[returns(ApprovalResponse)]
    IsApprovedForAll { owner: String, spender: String },
}

#[cw_serde]
pub struct NameResponse {
    pub name: String,
}

#[cw_serde]
pub struct SymbolResponse {
    pub symbol: String,
}

#[cw_serde]
pub struct TotalSupplyResponse {
    pub total_supply: Uint256,
}

#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint256,
}

#[cw_serde]
pub struct BalanceBatchResponse {
    pub balances: Vec<Uint256>,
}

#[cw_serde]
pub struct ApprovalResponse {
    pub approved: bool,
}
