mod example_data;

use cosmwasm_std::{Addr, ContractInfo};
use example_data::{ExampleData, VariousAddr, ACTIVE_ID, BIN_STEP};
use liquidity_book::interfaces::{
    lb_factory,
    lb_pair::{self, FactoryResponse},
    lb_router::*,
};
use shade_protocol::{swap::core::TokenType, utils::asset::RawContract};
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

// TODO: finish this

pub fn main() -> io::Result<()> {
    let package_name = "lb_router";

    // let crate_root = &env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut out_dir = env::current_dir().unwrap();
    out_dir.push("secretcli");
    create_dir_all(&out_dir).unwrap();

    let file_path = Path::new(&out_dir).join(format!("{package_name}.md"));
    let mut file = File::create(file_path.clone())?;

    writeln!(file, "# {package_name}\n")?;

    // -- Instantiate Message

    let instantiate_msg = InstantiateMsg {
        factory: ContractInfo::example(),
    };

    writeln!(file, "## Instantiate Message\n")?;
    print_instantiate_message!(file, instantiate_msg);

    // -- Execute Messages

    let create_lb_pair = ExecuteMsg::CreateLbPair {
        token_x: TokenType::example(),
        token_y: TokenType::example(),
        active_id: ACTIVE_ID,
        bin_step: BIN_STEP,
    };

    writeln!(file, "## Execute Messages\n")?;
    print_execute_messages!(file, create_lb_pair,);

    // -- Query Messages

    let get_factory = QueryMsg::GetFactory {};

    // responses

    let get_factory_response = FactoryResponse {
        factory: Addr::contract(),
    };

    writeln!(file, "## Query Messages with responses\n")?;
    print_query_messages_with_responses!(file, (get_factory, get_factory_response),);

    println!("Created {}", file_path.display());

    Ok(())
}
