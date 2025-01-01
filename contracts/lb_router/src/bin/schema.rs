use cosmwasm_schema::{export_schema, generate_api, remove_schemas, schema_for};
use liquidity_book::interfaces::lb_router::*;
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
//     // all the responses
// }
