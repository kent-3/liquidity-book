use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Event, Uint128};

#[derive(thiserror::Error, Debug)]
pub enum LbTokenError {
    #[error("AddressThisOrZero")]
    AddressThisOrZero,
    #[error("InvalidLength")]
    InvalidLength,
    #[error("SelfApproval: {0}")]
    SelfApproval(Addr),
    #[error("SpenderNotApproved: from {from}, spender {spender}")]
    SpenderNotApproved { from: Addr, spender: Addr },
    #[error("TransferExceedsBalance: from {from}, id {id}, amount {amount}")]
    TransferExceedsBalance {
        from: Addr,
        id: Uint128,
        amount: Uint128,
    },
    #[error("BurnExceedsBalance: from {from}, id {id}, amount {amount}")]
    BurnExceedsBalance {
        from: Addr,
        id: Uint128,
        amount: Uint128,
    },

    #[error(transparent)]
    StdError(#[from] cosmwasm_std::StdError),
}

pub trait LbTokenEventExt {
    fn transfer_batch(
        sender: Addr,
        from: Addr,
        to: Addr,
        ids: Vec<Uint128>,
        amounts: Vec<Uint128>,
    ) -> Event {
        Event::new("transfer_batch")
            .add_attribute_plaintext("sender", sender)
            .add_attribute_plaintext("from", from)
            .add_attribute_plaintext("to", to)
            .add_attribute_plaintext("ids", format!("{:?}", ids))
            .add_attribute_plaintext("amounts", format!("{:?}", amounts))
    }

    fn approval_for_all(account: Addr, sender: Addr, approved: bool) -> Event {
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
        spender: Addr,
        approved: bool,
    },
    BatchTransferFrom {
        from: Addr,
        to: Addr,
        ids: Vec<Uint128>,
        amounts: Vec<Uint128>,
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
    TotalSupply { id: Uint128 },
    #[returns(BalanceResponse)]
    BalanceOf { account: Addr, id: Uint128 },
    #[returns(BalanceBatchResponse)]
    BalanceOfBatch {
        accounts: Vec<Addr>,
        ids: Vec<Uint128>,
    },
    #[returns(ApprovalResponse)]
    IsApprovedForAll { owner: Addr, spender: Addr },
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
    pub total_supply: Uint128,
}

#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub struct BalanceBatchResponse {
    pub balances: Vec<Uint128>,
}

#[cw_serde]
pub struct ApprovalResponse {
    pub approved: bool,
}
