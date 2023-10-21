use cosmwasm_schema::write_api;

// TODO - derive QueryResponse trait for QueryMsg and QueryWithPermit
use lb_interfaces::lb_token::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryWithPermit};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        // query: QueryMsg,
    }
}
