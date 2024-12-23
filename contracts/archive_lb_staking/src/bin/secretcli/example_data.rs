#![allow(unused)]

use cosmwasm_std::{to_binary, Addr, ContractInfo, Uint128, Uint256, Uint64};
use lb_interfaces::{
    lb_factory::{ContractImplementation, StaticFeeParameters},
    lb_pair::{
        LbPair, LbPairInformation, LiquidityParameters, RemoveLiquidity, RewardsDistribution,
        TokenPair,
    },
    lb_staking::Auth,
    lb_token::Snip1155ReceiveMsg,
};
use serde_json::from_str;
use shade_protocol::{
    s_toolkit::permit::{Permit, TokenPermissions},
    snip20::Snip20ReceiveMsg,
    swap::core::{TokenAmount, TokenType},
    utils::asset::RawContract,
    Contract,
};

pub const BIN_STEP: u16 = 100u16;
pub const ACTIVE_ID: u32 = 8_388_608u32;
pub const DEFAULT_BASE_FACTOR: u16 = 5_000;
pub const DEFAULT_FILTER_PERIOD: u16 = 30;
pub const DEFAULT_DECAY_PERIOD: u16 = 600;
pub const DEFAULT_REDUCTION_FACTOR: u16 = 5_000;
pub const DEFAULT_VARIABLE_FEE_CONTROL: u32 = 40_000;
pub const DEFAULT_PROTOCOL_SHARE: u16 = 1_000;
pub const DEFAULT_MAX_VOLATILITY_ACCUMULATOR: u32 = 350_000;
pub trait VariousAddr {
    fn owner() -> Self;
    fn admin() -> Self;
    fn sender() -> Self;
    fn staker() -> Self;

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

    fn staker() -> Self {
        Addr::unchecked("secret1...staker")
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

impl ExampleData for ContractImplementation {
    fn example() -> Self {
        ContractImplementation {
            id: 1u64,
            code_hash: "0123456789ABCDEF".to_string(),
        }
        .clone()
    }
}

impl ExampleData for Auth {
    fn example() -> Self {
        Auth::ViewingKey {
            key: "viewing_key".to_string(),
            address: Addr::staker().to_string(),
        }
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

impl ExampleData for RawContract {
    fn example() -> Self {
        RawContract {
            address: Addr::contract().to_string(),
            code_hash: "0123456789ABCDEF".to_string(),
        }
        .clone()
    }
}

impl ExampleData for Contract {
    fn example() -> Self {
        Contract {
            address: Addr::contract(),
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

impl ExampleData for LiquidityParameters {
    fn example() -> Self {
        LiquidityParameters {
            token_x: TokenType::example(),
            token_y: TokenType::example(),
            bin_step: BIN_STEP,
            amount_x: Uint128::from(110u128),
            amount_y: Uint128::from(110u128),
            amount_x_min: Uint128::from(110u128),
            amount_y_min: Uint128::from(110u128),
            active_id_desired: ACTIVE_ID,
            // TODO - write some function that converts a price slippage % to an id_slippage (would
            // depend on bin_step)
            id_slippage: 1000u32,
            // TODO - I think these need to be much larger to hit the proper bin ids corresponding
            // to the next bin_step price.
            delta_ids: vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5],
            distribution_x: vec![Uint64::new(10u64); 11],
            distribution_y: vec![Uint64::new(10u64); 11],
            deadline: Uint64::new(1701283067u64),
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

impl ExampleData for RemoveLiquidity {
    fn example() -> Self {
        RemoveLiquidity {
            token_x: TokenType::example(),
            token_y: TokenType::example(),
            bin_step: BIN_STEP,
            amount_x_min: Uint128::from(10u128),
            amount_y_min: Uint128::from(10u128),
            ids: vec![ACTIVE_ID],
            // TODO - understand what "amounts" means. Is that a packed_uint128?
            amounts: vec![Uint256::from_u128(10u128)],
            deadline: 1701283067,
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

impl ExampleData for Snip1155ReceiveMsg {
    fn example() -> Self {
        Snip1155ReceiveMsg {
            sender: Addr::sender(),
            token_id: ACTIVE_ID.to_string(),
            from: Addr::sender(),
            amount: Uint256::default(),
            memo: None,
            msg: None,
        }
    }
}

impl ExampleData for RewardsDistribution {
    fn example() -> Self {
        RewardsDistribution {
            ids: vec![
                ACTIVE_ID - 4,
                ACTIVE_ID - 3,
                ACTIVE_ID - 2,
                ACTIVE_ID - 1,
                ACTIVE_ID,
                ACTIVE_ID + 1,
                ACTIVE_ID + 2,
                ACTIVE_ID + 3,
                ACTIVE_ID + 4,
                ACTIVE_ID + 5,
            ],
            weightages: vec![1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000, 1000],
            denominator: 10000,
        }
    }
}

impl ExampleData for TokenPair {
    fn example() -> Self {
        TokenPair(TokenType::example(), TokenType::example(), false)
    }
}

impl ExampleData for Permit<TokenPermissions> {
    fn example() -> Self {
        let json_str = r#"{
              "params": {
                  "permit_name": "Staking Query Permit",
                  "allowed_tokens": ["Staking Contract Permit"],
                  "chain_id": "secret-4",
                  "permissions": ["balance"]
              },
              "signature": {
                  "pub_key": {
                      "type": "tendermint/PubKeySecp256k1",
                      "value": "A5rpObgtxcUaEWK8lNj5plnc8a506yUTx9wswBguk5nl"
                  },
                  "signature": "y3QPwoSIPNlsMPDzoJfwgLxreSiHttrWfxXMusjavgR7F91tipEbDALAVe8XmPLX/f3GJBbA+nUQvkcT58B1SA=="
              }
          }"#;

        let permit: Permit<TokenPermissions> = from_str(json_str).expect("Failed to parse JSON");
        permit
    }
}
