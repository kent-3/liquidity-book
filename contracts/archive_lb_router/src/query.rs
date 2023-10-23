use cosmwasm_std::{to_binary, ContractInfo, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use lb_interfaces::lb_pair::{self, TokensResponse};

pub fn pair_contract_config(
    querier: &QuerierWrapper,
    pair_contract_address: ContractInfo,
) -> StdResult<TokensResponse> {
    let result: lb_pair::TokensResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract_address.address.to_string(),
        code_hash: pair_contract_address.code_hash,
        msg: to_binary(&lb_pair::QueryMsg::GetTokens {})?,
    }))?;

    Ok(result)
}
