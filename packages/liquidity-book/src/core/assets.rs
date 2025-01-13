use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, ContractInfo, StdResult};

/// In the process of being deprecated for [cosmwasm_std::ContractInfo] so use that
/// instead when possible.
#[derive(Hash, Eq)]
#[cw_serde]
pub struct Contract {
    pub address: Addr,
    pub code_hash: String,
}

impl Contract {
    #[allow(clippy::ptr_arg)]
    pub fn new(address: &Addr, code_hash: &String) -> Self {
        Contract {
            address: address.clone(),
            code_hash: code_hash.clone(),
        }
    }
}

/// A contract that does not contain a validated address.
/// Should be accepted as user input because we shouldn't assume addresses are verified Addrs.
/// https://docs.rs/cosmwasm-std/latest/cosmwasm_std/struct.Addr.html
#[derive(Hash, Eq, Default)]
#[cw_serde]
pub struct RawContract {
    pub address: String,
    pub code_hash: String,
}

impl RawContract {
    pub fn new(address: String, code_hash: String) -> Self {
        RawContract {
            address: address.clone(),
            code_hash: code_hash.clone(),
        }
    }

    /// Being deprecated in favor of `valid` which turns this into ContractInfo
    /// instead of a Contract (which we are getting rid of)
    pub fn into_valid(self, api: &dyn Api) -> StdResult<Contract> {
        let valid_addr = api.addr_validate(self.address.as_str())?;
        Ok(Contract::new(&valid_addr, &self.code_hash))
    }

    pub fn validate(self, api: &dyn Api) -> StdResult<ContractInfo> {
        let valid_addr = api.addr_validate(self.address.as_str())?;
        Ok(ContractInfo {
            address: valid_addr,
            code_hash: self.code_hash.clone(),
        })
    }
}

impl From<ContractInfo> for RawContract {
    fn from(item: ContractInfo) -> Self {
        RawContract {
            address: item.address.into(),
            code_hash: item.code_hash,
        }
    }
}

impl From<ContractInfo> for Contract {
    fn from(item: ContractInfo) -> Self {
        Contract {
            address: item.address,
            code_hash: item.code_hash,
        }
    }
}

impl From<Contract> for ContractInfo {
    fn from(item: Contract) -> ContractInfo {
        ContractInfo {
            address: item.address,
            code_hash: item.code_hash,
        }
    }
}

/// Validates a vector of Strings as Addrs
pub fn validate_vec(api: &dyn Api, unvalidated_addresses: Vec<String>) -> StdResult<Vec<Addr>> {
    let items: Result<Vec<_>, _> = unvalidated_addresses
        .iter()
        .map(|f| api.addr_validate(f.as_str()))
        .collect();
    Ok(items?)
}
