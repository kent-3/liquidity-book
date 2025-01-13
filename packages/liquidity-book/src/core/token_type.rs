use super::TokenAmount;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, ContractInfo, CosmosMsg, Deps, MessageInfo, StdError,
    StdResult, Uint128, WasmMsg,
};
use secret_toolkit::snip20;

// TODO: rename the fields!!!!!!!
#[cw_serde]
pub enum TokenType {
    CustomToken {
        contract_addr: Addr,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

#[cw_serde]
pub struct StableTokenData {
    pub oracle_key: String,
    pub decimals: u8,
}

#[cw_serde]
pub struct StableTokenType {
    pub token: TokenType,
    pub stable_token_data: StableTokenData,
}

impl TokenType {
    pub fn unique_key(&self) -> String {
        match self {
            TokenType::NativeToken { denom, .. } => denom.to_string(),
            TokenType::CustomToken { contract_addr, .. } => contract_addr.to_string(),
        }
    }

    pub fn is_custom_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => false,
            TokenType::CustomToken { .. } => true,
        }
    }

    pub fn is_native_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => true,
            TokenType::CustomToken { .. } => false,
        }
    }

    pub fn query_decimals(&self, deps: &Deps) -> StdResult<u8> {
        match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => Ok(snip20::token_info_query(
                deps.querier,
                0,
                token_code_hash.clone(),
                contract_addr.to_string(),
            )?
            .decimals),
            TokenType::NativeToken { denom } => match denom.as_str() {
                "uscrt" => Ok(6),
                _ => Err(StdError::generic_err(
                    "Cannot retrieve decimals for native token",
                )),
            },
        }
    }

    // TODO: return error if it's not a NativeToken
    pub fn assert_sent_native_token_balance(
        &self,
        info: &MessageInfo,
        amount: Uint128,
    ) -> StdResult<()> {
        if let TokenType::NativeToken { denom, .. } = &self {
            return match info.funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err(
                            "Native token balance mismatch between the argument and the transferred",
                        ))
                    }
                }
                None => {
                    if amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err(
                            "Native token balance mismatch between the argument and the transferred",
                        ))
                    }
                }
            };
        }

        Ok(())
    }

    pub fn new_amount(&self, amount: impl Into<Uint128> + Copy) -> TokenAmount {
        TokenAmount {
            token: self.clone(),
            amount: amount.into(),
        }
    }
}

impl From<ContractInfo> for TokenType {
    fn from(value: ContractInfo) -> Self {
        Self::CustomToken {
            contract_addr: value.address,
            token_code_hash: value.code_hash,
        }
    }
}

impl TokenType {
    pub fn query_balance(
        &self,
        deps: Deps,
        exchange_addr: String,
        viewing_key: String,
    ) -> StdResult<Uint128> {
        match self {
            TokenType::NativeToken { denom, .. } => {
                let result = deps.querier.query_balance(exchange_addr, denom)?;
                Ok(result.amount)
            }
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => snip20::balance_query(
                deps.querier,
                deps.api.addr_validate(&exchange_addr)?.to_string(),
                viewing_key,
                0,
                token_code_hash.clone(),
                contract_addr.to_string(),
            )
            .map(|balance| balance.amount),
        }
    }

    pub fn create_send_msg(&self, recipient: String, amount: Uint128) -> StdResult<CosmosMsg> {
        let msg = match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.clone().into_string(),
                code_hash: token_code_hash.to_string(),
                msg: to_binary(&snip20::HandleMsg::Send {
                    recipient,
                    amount,
                    padding: None,
                    msg: None,
                    recipient_code_hash: None,
                    memo: None,
                })?,
                funds: vec![],
            }),
            TokenType::NativeToken { denom, .. } => CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            }),
        };
        Ok(msg)
    }

    pub fn into_contract_info(&self) -> Option<ContractInfo> {
        match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
                ..
            } => Some(ContractInfo {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            }),
            TokenType::NativeToken { .. } => None,
        }
    }
}

// New Methods from LB

impl TokenType {
    // TODO: return Error instead of panic
    // In theory, native tokens do have an associated contract address, right? the wrapped version?
    pub fn address(&self) -> Addr {
        match self {
            TokenType::NativeToken { .. } => panic!("Doesn't work for native tokens"),
            TokenType::CustomToken { contract_addr, .. } => contract_addr.clone(),
        }
    }

    pub fn code_hash(&self) -> String {
        match self {
            TokenType::NativeToken { .. } => panic!("Doesn't work for native tokens"),
            TokenType::CustomToken {
                token_code_hash, ..
            } => token_code_hash.to_string(),
        }
    }

    pub fn transfer(&self, amount: Uint128, recipient: Addr) -> Option<CosmosMsg> {
        if amount.gt(&Uint128::zero()) {
            return None;
        }

        match &self {
            TokenType::CustomToken { .. } => {
                let msg = snip20::HandleMsg::Send {
                    recipient: recipient.to_string(),
                    amount,
                    padding: None,
                    msg: None,
                    recipient_code_hash: None,
                    memo: None,
                };

                msg.to_cosmos_msg(0, self.code_hash(), self.address().to_string(), None)
                    .ok()
            }

            TokenType::NativeToken { denom } => Some(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            })),
        }
    }

    pub fn transfer_from(
        &self,
        amount: Uint128,
        owner: Addr,
        recipient: Addr,
    ) -> Option<CosmosMsg> {
        if amount.gt(&Uint128::zero()) {
            return None;
        }

        match &self {
            TokenType::CustomToken { .. } => snip20::HandleMsg::TransferFrom {
                owner: owner.to_string(),
                recipient: recipient.to_string(),
                amount,
                padding: None,
                memo: None,
            }
            .to_cosmos_msg(0, self.code_hash(), self.address().to_string(), None)
            .ok(),

            TokenType::NativeToken { denom } => Some(CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            })),
        }
    }
}
