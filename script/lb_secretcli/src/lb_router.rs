mod example_data;

use cosmwasm_std::{Addr, ContractInfo, Uint128, Uint256, Uint64};
use example_data::{ExampleData, VariousAddr, ACTIVE_ID, BIN_STEP};
use liquidity_book::{
    core::{RawContract, TokenType},
    interfaces::{
        lb_factory,
        lb_pair::{self, FactoryResponse},
        lb_router::{self, *},
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

    let add_liquidity = ExecuteMsg::AddLiquidity {
        liquidity_parameters: LiquidityParameters::example(),
    };

    let swap_exact_tokens_for_tokens = ExecuteMsg::SwapExactTokensForTokens {
        amount_in: Uint128::new(1_000_000),
        amount_out_min: Uint128::new(950_000),
        path: lb_router::Path::example(),
        to: Addr::sender().to_string(),
        deadline: Uint64::new(1739317404),
    };

    writeln!(file, "## Execute Messages\n")?;
    print_execute_messages!(
        file,
        create_lb_pair,
        add_liquidity,
        swap_exact_tokens_for_tokens
    );

    // -- Query Messages with Responses

    let get_factory = QueryMsg::GetFactory {};
    let get_factory_response = FactoryResponse {
        factory: Addr::contract(),
    };

    let get_id_from_price = QueryMsg::GetIdFromPrice {
        lb_pair: ContractInfo::example(),
        price: Uint256::from(1_000_000_000_000_000_000u128),
    };
    let get_id_from_price_response = GetIdFromPriceResponse { id: 8_388_608u32 };

    let get_price_from_id = QueryMsg::GetPriceFromId {
        lb_pair: ContractInfo::example(),
        id: 8_388_608u32,
    };
    // NOTE: Price will be some monstrously large number
    // because it's actually a fixed point 128x128 number
    let get_price_from_id_response = GetPriceFromIdResponse {
        price: Uint256::from(1_000_000_000_000_000_000u128),
    };

    writeln!(file, "## Query Messages with responses\n")?;
    print_query_messages_with_responses!(
        file,
        (get_factory, get_factory_response),
        (get_id_from_price, get_id_from_price_response),
        (get_price_from_id, get_price_from_id_response)
    );

    println!("Created {}", file_path.display());

    Ok(())
}
