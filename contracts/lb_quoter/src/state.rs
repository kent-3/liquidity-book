use cosmwasm_std::ContractInfo;
use secret_toolkit::storage::Item;

pub const FACTORY_V2_2: Item<Option<ContractInfo>> = Item::new(b"factory_v2_2");
pub const ROUTER_V2_2: Item<Option<ContractInfo>> = Item::new(b"router_v2_2");
