mod example_data;

use cosmwasm_std::{Addr, ContractInfo};
use example_data::{ExampleData, VariousAddr, ACTIVE_ID, BIN_STEP};
use liquidity_book::{
    core::{RawContract, TokenType},
    interfaces::{lb_factory::*, lb_pair::LbPair},
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
    let package_name = "lb_factory";

    // let crate_root = &env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let mut out_dir = env::current_dir().unwrap();
    out_dir.push("secretcli");
    create_dir_all(&out_dir).unwrap();

    let file_path = Path::new(&out_dir).join(format!("{package_name}.md"));
    let mut file = File::create(file_path.clone())?;

    writeln!(file, "# {package_name}\n")?;

    // -- Instantiate Message

    let instantiate_msg = InstantiateMsg {
        admin_auth: RawContract::example(),
        owner: Some(Addr::owner()),
        fee_recipient: Addr::recipient(),
        query_auth: RawContract::example(),
    };

    writeln!(file, "## Instantiate Message\n")?;
    print_instantiate_message!(file, instantiate_msg);

    // -- Execute Messages

    let set_lb_pair_implementation = ExecuteMsg::SetLbPairImplementation {
        implementation: Implementation::example(),
    };

    let set_lb_token_implementation = ExecuteMsg::SetLbTokenImplementation {
        implementation: Implementation::example(),
    };

    let create_lb_pair = ExecuteMsg::CreateLbPair {
        token_x: TokenType::example(),
        token_y: TokenType::example(),
        // note: active id is a function of price
        active_id: ACTIVE_ID,
        bin_step: BIN_STEP,
        viewing_key: "api_key_etc".to_string(),
        entropy: "shade rocks".to_string(),
    };

    // TODO - find out what are more reasonable example values
    let set_pair_preset = ExecuteMsg::SetPreset {
        bin_step: BIN_STEP,
        base_factor: 100,
        filter_period: 100,
        decay_period: 100,
        reduction_factor: 100,
        variable_fee_control: 100,
        protocol_share: 100,
        max_volatility_accumulator: 100,
        is_open: true,
    };

    let set_preset_open_state = ExecuteMsg::SetPresetOpenState {
        bin_step: BIN_STEP,
        is_open: true,
    };

    let remove_preset = ExecuteMsg::RemovePreset { bin_step: BIN_STEP };

    let set_fee_parameters_on_pair = ExecuteMsg::SetFeeParametersOnPair {
        token_x: TokenType::example(),
        token_y: TokenType::example(),
        bin_step: BIN_STEP,
        base_factor: 100,
        filter_period: 100,
        decay_period: 100,
        reduction_factor: 100,
        variable_fee_control: 100,
        protocol_share: 100,
        max_volatility_accumulator: 100,
    };

    let set_fee_recipient = ExecuteMsg::SetFeeRecipient {
        fee_recipient: Addr::recipient(),
    };

    let add_quote_asset = ExecuteMsg::AddQuoteAsset {
        asset: TokenType::example(),
    };

    let remove_quote_asset = ExecuteMsg::RemoveQuoteAsset {
        asset: TokenType::example(),
    };

    let force_decay = ExecuteMsg::ForceDecay {
        pair: LbPair {
            token_x: TokenType::example(),
            token_y: TokenType::example(),
            bin_step: BIN_STEP,
            contract: ContractInfo::example(),
        },
    };

    writeln!(file, "## Execute Messages\n")?;
    print_execute_messages!(
        file,
        set_lb_pair_implementation,
        set_lb_token_implementation,
        create_lb_pair,
        set_pair_preset,
        set_preset_open_state,
        remove_preset,
        set_fee_parameters_on_pair,
        set_fee_recipient,
        add_quote_asset,
        remove_quote_asset,
        force_decay
    );

    // -- Query Messages

    let get_min_bin_step = QueryMsg::GetMinBinStep {};
    let get_fee_recipient = QueryMsg::GetFeeRecipient {};
    let get_lb_pair_implementation = QueryMsg::GetLbPairImplementation {};
    let get_lb_token_implementation = QueryMsg::GetLbTokenImplementation {};
    let get_number_of_lb_pairs = QueryMsg::GetNumberOfLbPairs {};
    let get_lb_pair_at_index = QueryMsg::GetLbPairAtIndex { index: 0 };
    let get_number_of_quote_assets = QueryMsg::GetNumberOfQuoteAssets {};
    let get_quote_asset_at_index = QueryMsg::GetQuoteAssetAtIndex { index: 0 };
    let is_quote_asset = QueryMsg::IsQuoteAsset {
        token: TokenType::example(),
    };
    let get_lb_pair_information = QueryMsg::GetLbPairInformation {
        token_x: TokenType::example(),
        token_y: TokenType::example(),
        bin_step: BIN_STEP,
    };
    let get_preset = QueryMsg::GetPreset { bin_step: BIN_STEP };
    let get_all_bin_steps = QueryMsg::GetAllBinSteps {};
    let get_open_bin_steps = QueryMsg::GetOpenBinSteps {};
    let get_all_lb_pairs = QueryMsg::GetAllLbPairs {
        token_x: TokenType::example(),
        token_y: TokenType::example(),
    };

    // responses

    let get_min_bin_step_response = MinBinStepResponse { min_bin_step: 100 };
    let get_fee_recipient_response = FeeRecipientResponse {
        fee_recipient: Addr::recipient(),
    };
    let get_lb_pair_implementation_response = LbPairImplementationResponse {
        lb_pair_implementation: Implementation::example(),
    };
    let get_lb_token_implementation_response = LbTokenImplementationResponse {
        lb_token_implementation: Implementation::example(),
    };
    let get_number_of_lb_pairs_response = NumberOfLbPairsResponse { lb_pair_number: 1 };

    let get_lb_pair_at_index_response = LbPairAtIndexResponse {
        lb_pair: LbPair {
            token_x: TokenType::example(),
            token_y: TokenType::example(),
            bin_step: 100,
            contract: ContractInfo::example(),
        },
    };

    let get_number_of_quote_assets_response = NumberOfQuoteAssetsResponse {
        number_of_quote_assets: 10,
    };

    let get_quote_asset_at_index_response = QuoteAssetAtIndexResponse {
        asset: TokenType::example(),
    };

    let is_quote_asset_response = IsQuoteAssetResponse { is_quote: true };

    let get_lb_pair_information_response = LbPairInformationResponse {
        lb_pair_information: LbPairInformation::example(),
    };

    let get_preset_response = PresetResponse {
        base_factor: 100,
        filter_period: 100,
        decay_period: 100,
        reduction_factor: 100,
        variable_fee_control: 100,
        protocol_share: 100,
        max_volatility_accumulator: 100,
        is_open: false,
    };

    let get_all_bin_steps_response = AllBinStepsResponse {
        bin_step_with_preset: vec![20, 50, 100],
    };

    let get_open_bin_steps_response = OpenBinStepsResponse {
        open_bin_steps: vec![20, 50, 100],
    };

    let get_all_lb_pairs_response = AllLbPairsResponse {
        lb_pairs_available: vec![LbPairInformation::example(), LbPairInformation::example()],
    };

    writeln!(file, "## Query Messages with responses\n")?;
    print_query_messages_with_responses!(
        file,
        (get_min_bin_step, get_min_bin_step_response),
        (get_fee_recipient, get_fee_recipient_response),
        (
            get_lb_pair_implementation,
            get_lb_pair_implementation_response
        ),
        (
            get_lb_token_implementation,
            get_lb_token_implementation_response
        ),
        (get_number_of_lb_pairs, get_number_of_lb_pairs_response),
        (get_lb_pair_at_index, get_lb_pair_at_index_response),
        (
            get_number_of_quote_assets,
            get_number_of_quote_assets_response
        ),
        (get_quote_asset_at_index, get_quote_asset_at_index_response),
        (is_quote_asset, is_quote_asset_response),
        (get_lb_pair_information, get_lb_pair_information_response),
        (get_preset, get_preset_response),
        (get_all_bin_steps, get_all_bin_steps_response),
        (get_open_bin_steps, get_open_bin_steps_response),
        (get_all_lb_pairs, get_all_lb_pairs_response),
    );

    println!("Created {}", file_path.display());

    Ok(())
}
