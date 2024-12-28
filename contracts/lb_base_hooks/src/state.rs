use cosmwasm_std::ContractInfo;
use secret_toolkit::storage::Item;

pub const LB_PAIR: Item<Option<ContractInfo>> = Item::new(b"linked_lb_pair");
