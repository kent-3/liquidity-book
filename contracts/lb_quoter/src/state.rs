use cosmwasm_schema::cw_serde;
use cosmwasm_std::ContractInfo;
use shade_protocol::secret_storage_plus::{Bincode2, Item};

pub const FACTORY_V2_2: Item<Option<ContractInfo>> = Item::new("factory_v2_2");
pub const ROUTER_V2_2: Item<Option<ContractInfo>> = Item::new("router_v2_2");
