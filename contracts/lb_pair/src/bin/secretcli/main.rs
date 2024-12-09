mod example_data;

use cosmwasm_std::{Addr, ContractInfo, Uint128, Uint256};
use ethnum::U256;
use example_data::*;
use lb_interfaces::{
    lb_factory::{ContractImplementation, StaticFeeParameters},
    lb_pair::*,
};
use lb_libraries::{math::uint256_to_u256::ConvertU256, pair_parameter_helper::PairParameters};
// TODO: any chance we can do away with these dependencies?
use shade_protocol::{
    swap::core::{TokenAmount, TokenType},
    utils::asset::RawContract,
};
use std::{
    env,
    fs::File,
    io::{self, Write},
    path::Path,
    str::FromStr,
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

fn main() -> io::Result<()> {
    let crate_root = &env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let package_name = &env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set");
    let file_path = Path::new(crate_root).join(format!("{package_name}.md"));
    let mut file = File::create(file_path.clone())?;

    writeln!(file, "# {package_name}\n")?;

    // -- Instantiate Message

    let mut preset = PairParameters::default();

    preset
        .set_static_fee_parameters(
            DEFAULT_BASE_FACTOR,
            DEFAULT_FILTER_PERIOD,
            DEFAULT_DECAY_PERIOD,
            DEFAULT_REDUCTION_FACTOR,
            DEFAULT_VARIABLE_FEE_CONTROL,
            DEFAULT_PROTOCOL_SHARE,
            DEFAULT_MAX_VOLATILITY_ACCUMULATOR,
        )
        .unwrap();

    let instantiate_msg = InstantiateMsg {
        admin_auth: RawContract::example(),
        factory: ContractInfo::example(),
        token_x: TokenType::example(),
        token_y: TokenType::example(),
        bin_step: 100,
        pair_parameters: StaticFeeParameters {
            base_factor: preset.get_base_factor(),
            filter_period: preset.get_filter_period(),
            decay_period: preset.get_decay_period(),
            reduction_factor: preset.get_reduction_factor(),
            variable_fee_control: preset.get_variable_fee_control(),
            protocol_share: preset.get_protocol_share(),
            max_volatility_accumulator: preset.get_max_volatility_accumulator(),
        },
        active_id: ACTIVE_ID,
        lb_token_implementation: ContractImplementation::default(),
        viewing_key: String::from("viewing_key"),
        entropy: String::from("entropy"),
        protocol_fee_recipient: Addr::funds_recipient(),
        query_auth: RawContract::example(),
    };

    writeln!(file, "## Instantiate Message\n")?;
    print_instantiate_message!(file, instantiate_msg);

    writeln!(file, "## Execute Messages\n")?;

    let add_liquidity = ExecuteMsg::AddLiquidity {
        liquidity_parameters: LiquidityParameters::example(),
    };

    let remove_liquidity = ExecuteMsg::RemoveLiquidity {
        remove_liquidity_params: RemoveLiquidity::example(),
    };

    let swap_tokens = ExecuteMsg::SwapTokens {
        offer: TokenAmount::example(),
        expected_return: None,
        to: Some(Addr::recipient().to_string()),
        padding: None,
    };

    let swap_tokens_invoke = InvokeMsg::SwapTokens {
        expected_return: None,
        to: Some(Addr::recipient().to_string()),
        padding: None,
    };

    let collect_protocol_fees = ExecuteMsg::CollectProtocolFees {};

    let set_static_fee_parameters = ExecuteMsg::SetStaticFeeParameters {
        base_factor: preset.get_base_factor(),
        filter_period: preset.get_filter_period(),
        decay_period: preset.get_decay_period(),
        reduction_factor: preset.get_reduction_factor(),
        variable_fee_control: preset.get_variable_fee_control(),
        protocol_share: preset.get_protocol_share(),
        max_volatility_accumulator: preset.get_max_volatility_accumulator(),
    };

    let force_decay = ExecuteMsg::ForceDecay {};

    let set_contract_status = ExecuteMsg::SetContractStatus {
        contract_status: ContractStatus::FreezeAll,
    };

    print_execute_messages!(
        file,
        add_liquidity,
        remove_liquidity,
        swap_tokens,
        swap_tokens_invoke,
        collect_protocol_fees,
        set_static_fee_parameters,
        force_decay,
        set_contract_status
    );

    // -- Query Messages

    writeln!(file, "## Query Messages with responses\n")?;

    // -- Query Messages
    let price: Uint256 = U256::from_str("42008768657166552252904831246223292524636112144")
        .unwrap()
        .u256_to_uint256();
    let total_liq = Uint256::from(100_000u128) * price + (Uint256::from(100_000u128) << 128);

    let get_factory = QueryMsg::GetFactory {};
    let get_token_x = QueryMsg::GetTokenX {};
    let get_token_y = QueryMsg::GetTokenY {};
    let get_bin_step = QueryMsg::GetBinStep {};
    let get_reserves = QueryMsg::GetReserves {};
    let get_active_id = QueryMsg::GetActiveId {};
    let get_bin_reserves = QueryMsg::GetBin { id: ACTIVE_ID };
    let get_next_non_empty_bin = QueryMsg::GetNextNonEmptyBin {
        swap_for_y: true,
        id: 1,
    };
    let get_protocol_fees = QueryMsg::GetProtocolFees {};
    let get_static_fee_parameters = QueryMsg::GetStaticFeeParameters {};
    let get_variable_fee_parameters = QueryMsg::GetVariableFeeParameters {};
    let get_oracle_parameters = QueryMsg::GetOracleParameters {};
    let get_oracle_sample_at = QueryMsg::GetOracleSampleAt {
        lookup_timestamp: 12345,
    };
    let get_price_from_id = QueryMsg::GetPriceFromId { id: ACTIVE_ID };

    let get_id_from_price = QueryMsg::GetIdFromPrice { price };
    let get_swap_in = QueryMsg::GetSwapIn {
        amount_out: Uint128::from(100_000u128),
        swap_for_y: true,
    };
    let get_swap_out = QueryMsg::GetSwapOut {
        amount_in: Uint128::from(100_000u128),
        swap_for_y: true,
    };

    // not in joe-v2
    let get_lb_token = QueryMsg::GetLbToken {};
    let get_lb_token_supply = QueryMsg::GetLbTokenSupply { id: 1 };
    let get_bins_reserves = QueryMsg::GetBins {
        ids: vec![ACTIVE_ID - 1, ACTIVE_ID, ACTIVE_ID + 1],
    };
    let get_all_bins = QueryMsg::GetAllBins {
        id: None,
        page: None,
        page_size: None,
    };

    // Responses

    let get_lb_token_response = LbTokenResponse {
        contract: ContractInfo::example(),
    };
    let get_factory_response = FactoryResponse {
        factory: Addr::contract(),
    };
    let get_token_x_response = TokenXResponse {
        token_x: TokenType::example(),
    };
    let get_token_y_response = TokenYResponse {
        token_y: TokenType::example(),
    };
    let get_bin_step_response = BinStepResponse { bin_step: 100 };
    let get_reserves_response = ReservesResponse {
        reserve_x: Uint128::new(1000),
        reserve_y: Uint128::new(1000),
    };
    let get_active_id_response = ActiveIdResponse {
        active_id: ACTIVE_ID,
    };
    let get_bin_reserves_response = BinResponse {
        bin_id: ACTIVE_ID,
        bin_reserve_x: Uint128::new(1000),
        bin_reserve_y: Uint128::new(1000),
    };

    let bin_responses = vec![
        BinResponse {
            bin_id: ACTIVE_ID - 1,
            bin_reserve_x: Uint128::new(1000),
            bin_reserve_y: Uint128::new(0),
        },
        BinResponse {
            bin_id: ACTIVE_ID,
            bin_reserve_x: Uint128::new(1000),
            bin_reserve_y: Uint128::new(1000),
        },
        BinResponse {
            bin_id: ACTIVE_ID + 1,
            bin_reserve_x: Uint128::new(0),
            bin_reserve_y: Uint128::new(1000),
        },
    ];
    let get_bins_reserves_response = BinsResponse(bin_responses.clone());
    let get_all_bins_reserves_response = AllBinsResponse {
        reserves: bin_responses.clone(),
        last_id: ACTIVE_ID + 1,
        current_block_height: 123456,
    };
    let get_next_non_empty_bin_response = NextNonEmptyBinResponse {
        next_id: ACTIVE_ID + 1,
    };
    let get_protocol_fees_response = ProtocolFeesResponse {
        protocol_fee_x: 1000,
        protocol_fee_y: 1000,
    };
    let get_static_fee_parameters_response = StaticFeeParametersResponse {
        base_factor: preset.get_base_factor(),
        filter_period: preset.get_filter_period(),
        decay_period: preset.get_decay_period(),
        reduction_factor: preset.get_reduction_factor(),
        variable_fee_control: preset.get_variable_fee_control(),
        protocol_share: preset.get_protocol_share(),
        max_volatility_accumulator: preset.get_max_volatility_accumulator(),
    };

    let get_variable_fee_parameters_response = VariableFeeParametersResponse {
        volatility_accumulator: preset.get_volatility_accumulator(),
        volatility_reference: preset.get_volatility_reference(),
        id_reference: preset.get_id_reference(),
        time_of_last_update: preset.get_time_of_last_update(),
    };

    let get_oracle_parameters_response = OracleParametersResponse {
        sample_lifetime: 120,
        size: 10,
        active_size: 10,
        last_updated: 1703403384,
        first_timestamp: 1703403383,
    };

    let get_oracle_sample_at_response = OracleSampleResponse {
        cumulative_id: 100,
        cumulative_volatility: 200,
        cumulative_bin_crossed: 50,
    };

    let get_price_from_id_response = PriceFromIdResponse { price };
    let get_id_from_price_response = IdFromPriceResponse { id: ACTIVE_ID };

    let get_swap_in_response = SwapInResponse {
        amount_in: Uint128::from(1000u128),
        amount_out_left: Uint128::from(10u128),
        fee: Uint128::from(10u128),
    };

    let get_swap_out_response = SwapOutResponse {
        amount_in_left: Uint128::from(1000u128),
        amount_out: Uint128::from(10u128),
        total_fees: Uint128::from(100u128),
        shade_dao_fees: Uint128::from(90u128),
        lp_fees: Uint128::from(10u128),
    };

    let get_lb_token_supply_response = LbTokenSupplyResponse {
        total_supply: total_liq,
    };

    print_query_messages_with_responses!(
        file,
        (get_lb_token, get_lb_token_response),
        (get_factory, get_factory_response),
        (get_token_x, get_token_x_response),
        (get_token_y, get_token_y_response),
        (get_bin_step, get_bin_step_response),
        (get_reserves, get_reserves_response),
        (get_active_id, get_active_id_response),
        (get_bin_reserves, get_bin_reserves_response),
        (get_bins_reserves, get_bins_reserves_response),
        (get_next_non_empty_bin, get_next_non_empty_bin_response),
        (get_protocol_fees, get_protocol_fees_response),
        (
            get_static_fee_parameters,
            get_static_fee_parameters_response
        ),
        (
            get_variable_fee_parameters,
            get_variable_fee_parameters_response
        ),
        (get_oracle_parameters, get_oracle_parameters_response),
        (get_oracle_sample_at, get_oracle_sample_at_response),
        (get_price_from_id, get_price_from_id_response),
        (get_id_from_price, get_id_from_price_response),
        (get_swap_in, get_swap_in_response),
        (get_swap_out, get_swap_out_response),
        (get_lb_token_supply, get_lb_token_supply_response),
    );

    println!("Created {}", file_path.display());

    Ok(())
}
