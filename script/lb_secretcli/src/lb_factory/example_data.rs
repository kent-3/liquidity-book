#![allow(unused)]

use cosmwasm_std::{to_binary, Addr, ContractInfo, Uint128, Uint256, Uint64};
use liquidity_book::interfaces::{
    lb_factory::{Implementation, LbPairInformation, StaticFeeParameters},
    lb_pair::LbPair,
};
use shade_protocol::{
    snip20::Snip20ReceiveMsg,
    swap::core::{TokenAmount, TokenType},
    utils::asset::RawContract,
};

pub const BIN_STEP: u16 = 100u16;
pub const ACTIVE_ID: u32 = 8_388_608u32;

pub trait VariousAddr {
    fn owner() -> Self;
    fn admin() -> Self;
    fn sender() -> Self;
    fn recipient() -> Self;
    fn funds_recipient() -> Self;
    fn contract() -> Self;
}

impl VariousAddr for Addr {
    fn owner() -> Self {
        Addr::unchecked("secret1...owner")
    }

    fn admin() -> Self {
        Addr::unchecked("secret1...admin")
    }

    fn sender() -> Self {
        Addr::unchecked("secret1...sender")
    }

    fn recipient() -> Self {
        Addr::unchecked("secret1...recipient")
    }

    fn funds_recipient() -> Self {
        Addr::unchecked("secret1...fundsrecipient")
    }

    fn contract() -> Self {
        Addr::unchecked("secret1...foobar")
    }
}

pub trait ExampleData {
    fn example() -> Self;
}

impl ExampleData for Implementation {
    fn example() -> Self {
        Implementation {
            id: 1u64,
            code_hash: "0123456789ABCDEF".to_string(),
        }
        .clone()
    }
}

impl ExampleData for TokenType {
    fn example() -> Self {
        TokenType::CustomToken {
            contract_addr: Addr::contract(),
            token_code_hash: "0123456789ABCDEF".to_string(),
        }
        .clone()
    }
}

impl ExampleData for TokenAmount {
    fn example() -> Self {
        TokenAmount {
            token: TokenType::example(),
            amount: Uint128::from(100u32),
        }
    }
}

impl ExampleData for ContractInfo {
    fn example() -> Self {
        ContractInfo {
            address: Addr::contract(),
            code_hash: "0123456789ABCDEF".to_string(),
        }
        .clone()
    }
}

// TODO - why are we using this instead of ContractInfo?
impl ExampleData for RawContract {
    fn example() -> Self {
        RawContract {
            address: Addr::contract().to_string(),
            code_hash: "0123456789ABCDEF".to_string(),
        }
        .clone()
    }
}

impl ExampleData for StaticFeeParameters {
    fn example() -> Self {
        StaticFeeParameters {
            base_factor: 100,
            filter_period: 100,
            decay_period: 100,
            reduction_factor: 100,
            variable_fee_control: 100,
            protocol_share: 100,
            max_volatility_accumulator: 100,
        }
    }
}

impl ExampleData for LbPairInformation {
    fn example() -> Self {
        LbPairInformation {
            bin_step: 100,
            lb_pair: LbPair {
                token_x: TokenType::example(),
                token_y: TokenType::example(),
                bin_step: 100,
                contract: ContractInfo::example(),
            },
            created_by_owner: true,
            ignored_for_routing: false,
        }
    }
}

impl ExampleData for Snip20ReceiveMsg {
    fn example() -> Self {
        Snip20ReceiveMsg {
            sender: Addr::contract().to_string(),
            from: Addr::sender().to_string(),
            amount: Uint128::from(100u128),
            memo: None,
            msg: Some(to_binary(&"base64 encoded string").unwrap()),
        }
    }
}
