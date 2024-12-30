// Testnet
pub static GRPC_URL: &str = "http://grpcbin.pulsar.scrttestnet.com:9099";
pub static CHAIN_ID: &str = "pulsar-3";
pub static MNEMONIC: &str = "";

// Devnet
// pub static GRPC_URL: &str = "http://localhost:9090";
// pub static CHAIN_ID: &str = "secretdev-1";
// pub static MNEMONIC: &str = "word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick";

pub static GAS_PRICE: u128 = 100_000;

// NOTE: This works, but feels like overkill. Could be used in CI maybe.

// use std::{
//     env,
//     sync::{Arc, LazyLock},
// };
// use tracing::info;
//
// pub static MNEMONIC: LazyLock<&str> = LazyLock::new(|| {
//     if let Ok(path) = dotenvy::dotenv() {
//         info!("Read .env file from {}", path.display());
//         let mnemonic = env::var("MNEMONIC").expect("No MNEMONIC set in .env");
//         Box::leak(mnemonic.into_boxed_str())
//     } else if let Ok(mnemonic) = env::var("MNEMONIC") {
//         info!("Read MNEMONIC from ENV");
//         Box::leak(mnemonic.into_boxed_str())
//     } else {
//         info!("Using pre-funded account D on localsecret");
//         "word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick"
//     }
// });
