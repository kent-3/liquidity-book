mod example_data;

use cosmwasm_std::{Addr, ContractInfo, Uint128};
use example_data::{ExampleData, VariousAddr, ACTIVE_ID, BIN_STEP};
use liquidity_book::{
    core::{RawContract, TokenType},
    interfaces::{
        lb_factory,
        lb_pair::{self, FactoryResponse},
        lb_quoter::*,
    },
};
use std::{
    env,
    fs::{create_dir_all, File},
    io::{self, Write},
    path::Path,
};

macro_rules! print_instantiate_message {
    ($file:ident, $($var:ident),+ $(,)?) => {
        $(
            writeln!($file,
                "```sh\nsecretcli tx compute instantiate 1 '{}'\n```",
                serde_json::to_string_pretty(&$var).unwrap()
            )?;
            writeln!($file, "")?;
        )+
    };
}

macro_rules! print_execute_messages {
    ($file:ident, $($var:ident),+ $(,)?) => {
        $(
            writeln!($file,
                "### {}\n\n```sh\nsecretcli tx compute execute secret1foobar '{}'\n```",
                stringify!($var),
                serde_json::to_string_pretty(&$var).unwrap()
            )?;
            writeln!($file, "")?;
        )+
    };
}

macro_rules! print_query_messages_with_responses {
      ($file:ident, $(($cmd:ident, $resp:ident)),+ $(,)?) => {
          $(
              writeln!($file,
                  "### {}\n\n```sh\nsecretcli query compute query secret1foobar '{}'\n```\n",
                  stringify!($cmd),
                  serde_json::to_string_pretty(&$cmd).unwrap()
              )?;
              writeln!($file,
                  "#### Response\n\n```json\n{}\n```\n",
                  serde_json::to_string_pretty(&$resp).unwrap()
              )?;
          )+
      };
  }

pub fn main() -> io::Result<()> {
    let package_name = "lb_quoter";

    // let crate_root = &env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut out_dir = env::current_dir().unwrap();
    out_dir.push("secretcli");
    create_dir_all(&out_dir).unwrap();

    let file_path = Path::new(&out_dir).join(format!("{package_name}.md"));
    let mut file = File::create(file_path.clone())?;

    writeln!(file, "# {package_name}\n")?;

    // -- Instantiate Message

    let instantiate_msg = InstantiateMsg {
        factory_v2_2: Some(RawContract::example()),
        router_v2_2: Some(RawContract::example()),
    };

    writeln!(file, "## Instantiate Message\n")?;
    print_instantiate_message!(file, instantiate_msg);

    // -- No Execute Messages

    // -- Query Messages and Responses

    let get_factory = QueryMsg::GetFactoryV2_2 {};
    let get_factory_response = FactoryV2_2Response {
        factory_v2_2: Some(ContractInfo::example()),
    };

    let get_router = QueryMsg::GetRouterV2_2 {};
    let get_router_response = RouterV2_2Response {
        router_v2_2: Some(ContractInfo::example()),
    };

    let find_best_path_from_amount_in = QueryMsg::FindBestPathFromAmountIn {
        route: vec![TokenType::example(), TokenType::example()],
        amount_in: Uint128::new(1_000_000),
    };
    let find_best_path_from_amount_in_response = Quote::example();

    // TODO: find_best_path_from_amount_out

    writeln!(file, "## Query Messages with responses\n")?;
    print_query_messages_with_responses!(
        file,
        (get_factory, get_factory_response),
        (get_router, get_router_response),
        (
            find_best_path_from_amount_in,
            find_best_path_from_amount_in_response
        )
    );

    println!("Created {}", file_path.display());

    Ok(())
}
