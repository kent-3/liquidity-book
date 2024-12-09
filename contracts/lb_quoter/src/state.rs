use cosmwasm_schema::cw_serde;
use cosmwasm_std::ContractInfo;
use shade_protocol::secret_storage_plus::{Bincode2, Item};

pub const STATE: Item<State> = Item::new("state");

#[cw_serde]
pub struct State {
    pub factory_v1: Option<ContractInfo>,
    pub router_v1: Option<ContractInfo>,
}
