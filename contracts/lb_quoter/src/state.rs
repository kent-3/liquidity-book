use cosmwasm_std::ContractInfo;
// TODO: switch to secret_toolkit_storage::Item
use shade_protocol::secret_storage_plus::Item;

pub const FACTORY_V2_2: Item<Option<ContractInfo>> = Item::new("factory_v2_2");
pub const ROUTER_V2_2: Item<Option<ContractInfo>> = Item::new("router_v2_2");
