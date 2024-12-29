use lb_interfaces::lb_pair::ILbPair;
use secret_toolkit::storage::Item;

pub const LB_PAIR: Item<Option<ILbPair>> = Item::new(b"lb_pair");
