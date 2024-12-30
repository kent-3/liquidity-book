#![allow(unused)]

// .truecolor(255, 160, 10)

mod account;
mod constants;
mod support;
mod utils;

use crate::{
    account::Account,
    constants::{CHAIN_ID, GRPC_URL, MNEMONIC},
    support::snip20,
    utils::{check_gas, code_hash_by_code_id, execute, instantiate, sha256, store_code},
};
use color_eyre::{owo_colors::OwoColorize, Result};
use cosmwasm_std::{to_binary, Addr, Binary, ContractInfo, Uint128, Uint64};
use ethnum::U256;
use liquidity_book::{
    interfaces::{
        lb_factory,
        lb_pair::{self, LbPair},
        lb_quoter,
        lb_router::{self, AddLiquidityResponse, CreateLbPairResponse},
    },
    libraries::{
        error::U256x256MathError, math::uint256_to_u256::ConvertUint256, price_helper::PriceHelper,
    },
};
use secretrs::{
    grpc_clients::{AuthQueryClient, ComputeQueryClient, RegistrationQueryClient, TxServiceClient},
    utils::EnigmaUtils,
};
use serde::{Deserialize, Serialize};
use shade_protocol::{swap::core::TokenType, utils::asset::RawContract, Contract};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
    time::Duration,
};
use tonic::transport::Channel;
use tracing::{debug, info, info_span};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

#[derive(Debug)]
pub struct Secret<T> {
    pub wallet: Account,
    pub utils: EnigmaUtils,
    pub auth: AuthQueryClient<T>,
    pub compute: ComputeQueryClient<T>,
    pub tx: TxServiceClient<T>,
}

static SECRET: OnceLock<Secret<Channel>> = OnceLock::new();

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    ::color_eyre::install()?;

    let filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::INFO.into()) // Default level for other crates
        .add_directive("lb_deployer=INFO".parse().unwrap()); // level for this crate

    ::tracing_subscriber::fmt()
        .with_env_filter(filter)
        // .pretty()
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .init();

    let channel = Channel::builder(GRPC_URL.parse()?)
        .timeout(Duration::from_secs(60))
        .connect()
        .await?;
    let secretrs = setup_client(channel).await?;
    let wallet_address = secretrs.wallet.addr();

    // Store Code
    let admin = Path::new("./code/admin.wasm.gz");
    let query_auth = Path::new("./code/query_auth.wasm.gz");
    let query_router = Path::new("./code/query_router.wasm");
    let snip20 = Path::new("./code/snip20.wasm.gz");
    let snip25 = Path::new("./code/snip25-amber.wasm.gz");
    let lb_factory = Path::new("../liquidity-book/code/lb_factory.wasm.gz");
    let lb_pair = Path::new("../liquidity-book/code/lb_pair.wasm.gz");
    let lb_token = Path::new("../liquidity-book/code/lb_token.wasm.gz");
    let lb_router = Path::new("../liquidity-book/code/lb_router.wasm.gz");
    let lb_quoter = Path::new("../liquidity-book/code/lb_quoter.wasm.gz");

    let admin_code_id = store_code(admin, 1_000_000).await?;
    let query_auth_code_id = store_code(query_auth, 1_400_000).await?;
    let query_router_code_id = store_code(query_router, 1_700_000).await?;
    let snip20_code_id = store_code(snip20, 1_200_000).await?;
    let snip25_code_id = store_code(snip25, 2_900_000).await?;
    let lb_factory_code_id = store_code(lb_factory, 2_100_000).await?;
    let lb_pair_code_id = store_code(lb_pair, 3_600_000).await?;
    let lb_token_code_id = store_code(lb_token, 2_600_000).await?;
    let lb_router_code_id = store_code(lb_router, 2_100_000).await?;
    let lb_quoter_code_id = store_code(lb_quoter, 1_000_000).await?;

    info!("Gas used to store codes: {}", check_gas());

    // TODO: hash the code directly
    let admin_code_hash = code_hash_by_code_id(admin_code_id).await?;
    let query_auth_code_hash = code_hash_by_code_id(query_auth_code_id).await?;
    let query_router_code_hash = code_hash_by_code_id(query_router_code_id).await?;
    let snip20_code_hash = code_hash_by_code_id(snip20_code_id).await?;
    let snip25_code_hash = code_hash_by_code_id(snip25_code_id).await?;
    let lb_factory_code_hash = code_hash_by_code_id(lb_factory_code_id).await?;
    let lb_pair_code_hash = code_hash_by_code_id(lb_pair_code_id).await?;
    let lb_token_code_hash = code_hash_by_code_id(lb_token_code_id).await?;
    let lb_router_code_hash = code_hash_by_code_id(lb_router_code_id).await?;
    let lb_quoter_code_hash = code_hash_by_code_id(lb_quoter_code_id).await?;

    // Instantiate

    info!("Instantiating admin...",);
    let admin_init_msg = shade_protocol::contract_interfaces::admin::InstantiateMsg {
        super_admin: Some(wallet_address.to_string()),
    };
    let admin = instantiate(admin_code_id, &admin_code_hash, &admin_init_msg, 100_000).await?;

    info!("Instantiating query_auth...",);
    let query_auth_init_msg = shade_protocol::contract_interfaces::query_auth::InstantiateMsg {
        admin_auth: Contract {
            address: Addr::unchecked(admin.address.clone()),
            code_hash: admin.code_hash.clone(),
        },
        prng_seed: Binary([1u8; 32].to_vec()),
    };
    let query_auth = instantiate(
        query_auth_code_id,
        &query_auth_code_hash,
        &query_auth_init_msg,
        100_000,
    )
    .await?;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
    pub struct EmptyInstantiateMsg {}

    info!("Instantiating batch_query_router...",);
    let query_router_init_msg = EmptyInstantiateMsg {};
    let query_router = instantiate(
        query_router_code_id,
        &query_router_code_hash,
        &query_router_init_msg,
        100_000,
    )
    .await?;

    info!("Instantiating lb_factory...",);
    let lb_factory_init_msg = lb_factory::InstantiateMsg {
        // TODO is it meant to be admin_auth or admin?
        admin_auth: RawContract {
            address: admin.address.to_string(),
            code_hash: admin.code_hash.to_string(),
        },
        query_auth: RawContract {
            address: query_auth.address.to_string(),
            code_hash: query_auth.code_hash.to_string(),
        },
        owner: Some(wallet_address.clone()),
        fee_recipient: wallet_address.clone(),
    };
    let lb_factory = instantiate(
        lb_factory_code_id,
        &lb_factory_code_hash,
        &lb_factory_init_msg,
        100_000,
    )
    .await?;

    info!("Instantiating lb_router...",);
    let lb_router_init_msg = lb_router::InstantiateMsg {
        factory: lb_factory.clone(),
    };
    let lb_router = instantiate(
        lb_router_code_id,
        &lb_router_code_hash,
        &lb_router_init_msg,
        100_000,
    )
    .await?;

    info!("Instantiating lb_quoter...",);
    let lb_quoter_init_msg = lb_quoter::InstantiateMsg {
        factory_v2_2: Some(RawContract {
            address: lb_factory.address.clone().to_string(),
            code_hash: lb_factory.code_hash.clone(),
        }),
        router_v2_2: Some(RawContract {
            address: lb_router.address.clone().to_string(),
            code_hash: lb_router.code_hash.clone(),
        }),
    };
    let lb_quoter = instantiate(
        lb_quoter_code_id,
        &lb_quoter_code_hash,
        &lb_quoter_init_msg,
        100_000,
    )
    .await?;

    // Make 2 Tokens

    info!("Instantiating snip20...",);
    let snip20_init_msg = snip20::InstantiateMsg {
        name: "Secret Secret".to_string(),
        admin: None,
        symbol: "SSCRT".to_string(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            address: wallet_address.to_string(),
            amount: Uint128::MAX,
        }]),
        prng_seed: to_binary(b"secret_rocks")?,
        config: None,
        supported_denoms: Some(vec!["uscrt".to_string()]),
    };
    let snip20 = instantiate(snip20_code_id, &snip20_code_hash, &snip20_init_msg, 100_000).await?;

    info!("Instantiating snip25...",);
    let snip25_init_msg = snip20::InstantiateMsg {
        name: "Amber".to_string(),
        admin: None,
        symbol: "AMBER".to_string(),
        decimals: 6,
        initial_balances: Some(vec![snip20::InitialBalance {
            address: wallet_address.to_string(),
            amount: Uint128::MAX,
        }]),
        prng_seed: to_binary(b"amber_rocks")?,
        config: None,
        supported_denoms: None,
    };
    let snip25 = instantiate(snip25_code_id, &snip25_code_hash, &snip25_init_msg, 100_000).await?;

    // Factory Setup

    let address = lb_factory.address.as_str();
    let code_hash = lb_factory_code_hash.as_str();

    // Tell the Factory which codes to use when creating contracts.
    let set_lb_pair_implementation_msg = &lb_factory::ExecuteMsg::SetLbPairImplementation {
        implementation: lb_factory::Implementation {
            id: lb_pair_code_id,
            code_hash: lb_pair_code_hash.to_string(),
        },
    };
    let set_lb_token_implementation_msg = &lb_factory::ExecuteMsg::SetLbTokenImplementation {
        implementation: lb_factory::Implementation {
            id: lb_token_code_id,
            code_hash: lb_token_code_hash.to_string(),
        },
    };

    info!("Setting lb_pair implementation...",);
    execute(address, code_hash, set_lb_pair_implementation_msg, 100_000).await?;

    info!("Setting lb_token implementation...",);
    execute(address, code_hash, set_lb_token_implementation_msg, 100_000).await?;

    // TODO: determine sensible values
    let set_pair_preset_msg = &lb_factory::ExecuteMsg::SetPreset {
        bin_step: 100,
        base_factor: 5_000,
        filter_period: 30,
        decay_period: 600,
        reduction_factor: 5_000,
        variable_fee_control: 5_000,
        protocol_share: 0,
        max_volatility_accumulator: 350_000,
        // sample_lifetime: 120,
        is_open: true,
    };
    info!("Setting pair presets for bin_step = 100...",);
    execute(address, code_hash, set_pair_preset_msg, 100_000).await?;

    let add_quote_asset_msg = &lb_factory::ExecuteMsg::AddQuoteAsset {
        asset: TokenType::CustomToken {
            contract_addr: Addr::unchecked(snip20.address.as_str()),
            token_code_hash: snip20_code_hash.to_string(),
        },
    };
    info!("Adding sSCRT as a quote asset...",);
    execute(address, code_hash, add_quote_asset_msg, 100_000).await?;

    // Use the router to create a pair
    // this ensures the viewing key is always the same

    let router_address = lb_router.address.as_str();
    let router_code_hash = lb_router_code_hash.as_str();

    let create_lb_pair_msg = &lb_router::ExecuteMsg::CreateLbPair {
        token_x: TokenType::CustomToken {
            contract_addr: Addr::unchecked(snip25.address.as_str()),
            token_code_hash: snip25_code_hash.to_string(),
        },
        token_y: TokenType::CustomToken {
            contract_addr: Addr::unchecked(snip20.address.as_str()),
            token_code_hash: snip20_code_hash.to_string(),
        },
        active_id: 8_388_608,
        bin_step: 100,
        // viewing_key: "lb_rocks".to_string(),
        // entropy: "lb_rocks".to_string(),
    };

    info!("Creating an Lb Pair...",);
    let response = execute(
        router_address,
        router_code_hash,
        create_lb_pair_msg,
        700_000,
    )
    .await?;

    let created_lb_pair = serde_json::from_slice::<CreateLbPairResponse>(&response)?.lb_pair;
    info!("{:#?}", created_lb_pair);

    let pair_address = created_lb_pair.contract.address.as_str();
    let pair_code_hash = created_lb_pair.contract.code_hash.as_str();

    // WARN: It's going to cost a lot to increase the oracle to full length (65535)!
    let increase_length_msg = &lb_pair::ExecuteMsg::IncreaseOracleLength { new_length: 2500 };

    info!("Increasing Oracle length...",);
    let response = execute(pair_address, pair_code_hash, increase_length_msg, 5_000_000).await?;

    // add liquidity

    let increase_allowance_msg = &secret_toolkit_snip20::HandleMsg::IncreaseAllowance {
        spender: router_address.to_owned(),
        amount: Uint128::MAX,
        expiration: None,
        padding: None,
    };

    info!("Increasing token_x allowance...",);
    let response = execute(
        created_lb_pair.token_x.address().as_str(),
        created_lb_pair.token_x.code_hash().as_str(),
        increase_allowance_msg,
        5_000_000,
    )
    .await?;

    info!("Increasing token_y allowance...",);
    let response = execute(
        created_lb_pair.token_y.address().as_str(),
        created_lb_pair.token_y.code_hash().as_str(),
        increase_allowance_msg,
        5_000_000,
    )
    .await?;

    let mut liquidity_parameters = lb_pair::LiquidityParameters {
        token_x: created_lb_pair.token_x.clone(),
        token_y: created_lb_pair.token_y.clone(),
        bin_step: 100,
        amount_x: Uint128::from(1000000u128),
        amount_y: Uint128::from(1000000u128),
        amount_x_min: Uint128::from(950000u128),
        amount_y_min: Uint128::from(950000u128),
        active_id_desired: 8388608,
        id_slippage: 10,
        delta_ids: vec![-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5],
        distribution_x: vec![
            Uint64::new(0),
            Uint64::new(0),
            Uint64::new(0),
            Uint64::new(0),
            Uint64::new(0),
            Uint64::new(90909000000000000),
            Uint64::new(181818000000000000),
            Uint64::new(181818000000000000),
            Uint64::new(181818000000000000),
            Uint64::new(181818000000000000),
            Uint64::new(181818000000000000),
        ],
        distribution_y: vec![
            Uint64::new(181818000000000000),
            Uint64::new(181818000000000000),
            Uint64::new(181818000000000000),
            Uint64::new(181818000000000000),
            Uint64::new(181818000000000000),
            Uint64::new(90909000000000000),
            Uint64::new(0),
            Uint64::new(0),
            Uint64::new(0),
            Uint64::new(0),
            Uint64::new(0),
        ],
        to: wallet_address.to_string(),
        refund_to: wallet_address.to_string(),
        deadline: Uint64::new(2000000000),
    };

    debug!("{:#?}", liquidity_parameters);

    let add_liquidity_msg = &lb_router::ExecuteMsg::AddLiquidity {
        liquidity_parameters: liquidity_parameters.clone(),
    };

    info!("Adding Liquidity...",);
    let response = execute(
        router_address,
        router_code_hash,
        add_liquidity_msg,
        1_000_000,
    )
    .await
    .inspect_err(|e| info!("{e}"));

    if let Ok(ref response) = response {
        if let Ok(add_liquidity_response) = serde_json::from_slice::<AddLiquidityResponse>(response)
        {
            info!("{:#?}", add_liquidity_response);
            let converted_liquidity = PriceHelper::convert128x128_price_to_decimal(
                add_liquidity_response.liquidity_minted[0].uint256_to_u256(),
            );
            let converted_liquidity: Vec<U256> = add_liquidity_response
                .liquidity_minted
                .iter()
                .map(|el| PriceHelper::convert128x128_price_to_decimal(el.uint256_to_u256()))
                .collect::<Result<_, U256x256MathError>>()?;
            info!("{:#?}", converted_liquidity);
        }
    }

    // wrapping up

    let lb_pair = ContractInfo {
        address: created_lb_pair.contract.address,
        code_hash: created_lb_pair.contract.code_hash,
    };
    let lb_token = ContractInfo {
        address: Addr::unchecked(""),
        code_hash: lb_token_code_hash.to_string(),
    };

    let deployment = DeployedContracts {
        admin_auth: DeployedContractInfo {
            address: admin.address,
            code_hash: admin.code_hash,
            code_id: admin_code_id,
        },
        query_auth: DeployedContractInfo {
            address: query_auth.address,
            code_hash: query_auth.code_hash,
            code_id: query_auth_code_id,
        },
        snip20: DeployedContractInfo {
            address: snip20.address,
            code_hash: snip20.code_hash,
            code_id: snip20_code_id,
        },
        snip25: DeployedContractInfo {
            address: snip25.address,
            code_hash: snip25.code_hash,
            code_id: snip25_code_id,
        },
        lb_factory: DeployedContractInfo {
            address: lb_factory.address,
            code_hash: lb_factory.code_hash,
            code_id: lb_factory_code_id,
        },
        lb_pair: DeployedContractInfo {
            address: lb_pair.address,
            code_hash: lb_pair.code_hash,
            code_id: lb_pair_code_id,
        },
        lb_token: DeployedContractInfo {
            address: lb_token.address,
            code_hash: lb_token.code_hash,
            code_id: lb_token_code_id,
        },
        lb_router: DeployedContractInfo {
            address: lb_router.address,
            code_hash: lb_router.code_hash,
            code_id: lb_router_code_id,
        },
        lb_quoter: DeployedContractInfo {
            address: lb_quoter.address,
            code_hash: lb_quoter.code_hash,
            code_id: lb_quoter_code_id,
        },
    };

    debug!("{:#?}", deployment);

    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let serialized = serde_json::to_string(&deployment).expect("Failed to serialize deployment");
    // TODO: replace if with match
    let map_file_path = if CHAIN_ID == "secretdev-1" {
        out_dir.join("lb_contracts_dev.json")
    } else if CHAIN_ID == "pulsar-3" {
        out_dir.join("lb_contracts_pulsar.json")
    } else {
        out_dir.join("lb_contracts.json")
    };
    fs::write(&map_file_path, serialized).expect("Failed to write lb_contracts json file!");

    info!("Deployment details saved to {}", map_file_path.display());

    info!("Total gas used: {}", check_gas());

    Ok(())
}

pub async fn setup_client(
    channel: tonic::transport::Channel,
) -> Result<&'static Secret<tonic::transport::Channel>> {
    let mut secret_registration = RegistrationQueryClient::new(channel.clone());
    let enclave_key_bytes = secret_registration.tx_key(()).await?.into_inner().key;
    let enclave_key = hex::encode(&enclave_key_bytes);
    info!("Enclave IO Public Key: {:>4}", enclave_key.bright_blue());

    let mut enclave_key = [0u8; 32];
    enclave_key.copy_from_slice(&enclave_key_bytes[0..32]);

    let wallet = Account::from_mnemonic(&MNEMONIC).expect("Failed to parse mnemonic");
    let wallet_address = wallet.addr();
    // TODO: figure out a more secure seed
    let seed = sha256(wallet.addr().as_bytes());
    let utils = EnigmaUtils::from_io_key(Some(seed), enclave_key);

    let secretrs = SECRET.get_or_init(|| Secret {
        wallet,
        utils,
        auth: AuthQueryClient::new(channel.clone()),
        compute: ComputeQueryClient::new(channel.clone()),
        tx: TxServiceClient::new(channel.clone()),
    });

    info!(
        "Initialized client with wallet address: {}",
        &wallet_address
    );
    info!("Connected to {}\n", GRPC_URL);

    debug!(
        "Wallet encryption utils seed: {}",
        hex::encode(secretrs.utils.get_seed())
    );

    Ok(secretrs)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContractInfo {
    pub address: Addr,
    pub code_hash: String,
    pub code_id: u64,
}

impl Default for DeployedContractInfo {
    fn default() -> Self {
        Self {
            address: Addr::unchecked(""),
            code_hash: "".to_string(),
            code_id: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContracts {
    pub admin_auth: DeployedContractInfo,
    pub query_auth: DeployedContractInfo,
    pub snip20: DeployedContractInfo,
    pub snip25: DeployedContractInfo,
    pub lb_factory: DeployedContractInfo,
    pub lb_pair: DeployedContractInfo,
    pub lb_token: DeployedContractInfo,
    pub lb_router: DeployedContractInfo,
    pub lb_quoter: DeployedContractInfo,
}

impl DeployedContracts {
    pub fn new() -> Self {
        DeployedContracts {
            admin_auth: DeployedContractInfo::default(),
            query_auth: DeployedContractInfo::default(),
            snip20: DeployedContractInfo::default(),
            snip25: DeployedContractInfo::default(),
            lb_factory: DeployedContractInfo::default(),
            lb_pair: DeployedContractInfo::default(),
            lb_token: DeployedContractInfo::default(),
            lb_router: DeployedContractInfo::default(),
            lb_quoter: DeployedContractInfo::default(),
        }
    }
}
