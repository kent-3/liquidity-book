use cosmwasm_schema::{export_schema, generate_api, remove_schemas, schema_for};
use liquidity_book::interfaces::lb_factory::{
    AllBinStepsResponse, AllLbPairsResponse, ExecuteMsg, FeeRecipientResponse, InstantiateMsg,
    IsQuoteAssetResponse, LbPairAtIndexResponse, LbPairImplementationResponse,
    LbPairInformationResponse, LbTokenImplementationResponse, MinBinStepResponse,
    NumberOfLbPairsResponse, NumberOfQuoteAssetsResponse, OpenBinStepsResponse, PresetResponse,
    QueryMsg, QuoteAssetAtIndexResponse,
};
use std::{
    env,
    fs::{create_dir_all, write},
    path::PathBuf,
};

fn main() {
    // Get the directory of the current crate
    let mut out_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    let api = generate_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
    .render();

    let path = out_dir.join(concat!("lb_pair", ".json"));

    let json = api.to_string().unwrap();
    write(&path, json + "\n").unwrap();
    println!("Exported the full API as {}", path.to_str().unwrap());

    let raw_dir = out_dir.join("raw");
    create_dir_all(&raw_dir).unwrap();

    for (filename, json) in api.to_schema_files().unwrap() {
        let path = raw_dir.join(filename);

        write(&path, json + "\n").unwrap();
        println!("Exported {}", path.to_str().unwrap());
    }
}

// fn main() {
//     // Get the directory of the current crate
//     let mut out_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
//     out_dir.push("schema");
//
//     create_dir_all(&out_dir).unwrap();
//     remove_schemas(&out_dir).unwrap();
//
//     export_schema(&schema_for!(InstantiateMsg), &out_dir);
//     export_schema(&schema_for!(ExecuteMsg), &out_dir);
//     export_schema(&schema_for!(QueryMsg), &out_dir);
//
//     // Add export_schema for each response struct
//     export_schema(&schema_for!(MinBinStepResponse), &out_dir);
//     export_schema(&schema_for!(FeeRecipientResponse), &out_dir);
//     export_schema(&schema_for!(LbPairImplementationResponse), &out_dir);
//     export_schema(&schema_for!(LbTokenImplementationResponse), &out_dir);
//     export_schema(&schema_for!(NumberOfLbPairsResponse), &out_dir);
//     export_schema(&schema_for!(LbPairAtIndexResponse), &out_dir);
//     export_schema(&schema_for!(NumberOfQuoteAssetsResponse), &out_dir);
//     export_schema(&schema_for!(QuoteAssetAtIndexResponse), &out_dir);
//     export_schema(&schema_for!(IsQuoteAssetResponse), &out_dir);
//     export_schema(&schema_for!(LbPairInformationResponse), &out_dir);
//     export_schema(&schema_for!(PresetResponse), &out_dir);
//     export_schema(&schema_for!(AllBinStepsResponse), &out_dir);
//     export_schema(&schema_for!(OpenBinStepsResponse), &out_dir);
//     export_schema(&schema_for!(AllLbPairsResponse), &out_dir);
// }
