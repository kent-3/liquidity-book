use cosmwasm_schema::{export_schema, generate_api, remove_schemas, schema_for};
use liquidity_book::interfaces::*;
use std::{
    env,
    fs::{create_dir_all, write},
    path::PathBuf,
};

// TODO: less brute force loop

fn main() {
    let mut out_dir = env::current_dir().unwrap();

    out_dir.push("schema/lb_factory");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    let factory_api = generate_api! {
        instantiate: lb_factory::InstantiateMsg,
        execute: lb_factory::ExecuteMsg,
        query: lb_factory::QueryMsg,
    }
    .render();

    let path = out_dir.join(concat!("lb_factory", ".json"));

    let json = factory_api.to_string().unwrap();
    write(&path, json + "\n").unwrap();
    println!("Exported the full API as {}", path.to_str().unwrap());

    let raw_dir = out_dir.join("raw");
    create_dir_all(&raw_dir).unwrap();

    for (filename, json) in factory_api.to_schema_files().unwrap() {
        let path = raw_dir.join(filename);

        write(&path, json + "\n").unwrap();
        println!("Exported {}", path.to_str().unwrap());
    }

    // ---

    let mut out_dir = env::current_dir().unwrap();

    out_dir.push("schema/lb_pair");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    let pair_api = generate_api! {
        instantiate: lb_pair::InstantiateMsg,
        execute: lb_pair::ExecuteMsg,
        query: lb_pair::QueryMsg,
    }
    .render();

    let path = out_dir.join(concat!("lb_pair", ".json"));

    let json = pair_api.to_string().unwrap();
    write(&path, json + "\n").unwrap();
    println!("Exported the full API as {}", path.to_str().unwrap());

    let raw_dir = out_dir.join("raw");
    create_dir_all(&raw_dir).unwrap();

    for (filename, json) in pair_api.to_schema_files().unwrap() {
        let path = raw_dir.join(filename);

        write(&path, json + "\n").unwrap();
        println!("Exported {}", path.to_str().unwrap());
    }

    // ---

    let mut out_dir = env::current_dir().unwrap();

    out_dir.push("schema/lb_quoter");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    let quoter_api = generate_api! {
        instantiate: lb_quoter::InstantiateMsg,
        execute: lb_quoter::ExecuteMsg,
        query: lb_quoter::QueryMsg,
    }
    .render();

    let path = out_dir.join(concat!("lb_quoter", ".json"));

    let json = quoter_api.to_string().unwrap();
    write(&path, json + "\n").unwrap();
    println!("Exported the full API as {}", path.to_str().unwrap());

    let raw_dir = out_dir.join("raw");
    create_dir_all(&raw_dir).unwrap();

    for (filename, json) in quoter_api.to_schema_files().unwrap() {
        let path = raw_dir.join(filename);

        write(&path, json + "\n").unwrap();
        println!("Exported {}", path.to_str().unwrap());
    }

    // ---

    let mut out_dir = env::current_dir().unwrap();

    out_dir.push("schema/lb_router");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    let router_api = generate_api! {
        instantiate: lb_router::InstantiateMsg,
        execute: lb_router::ExecuteMsg,
        query: lb_router::QueryMsg,
    }
    .render();

    let path = out_dir.join(concat!("lb_router", ".json"));

    let json = router_api.to_string().unwrap();
    write(&path, json + "\n").unwrap();
    println!("Exported the full API as {}", path.to_str().unwrap());

    let raw_dir = out_dir.join("raw");
    create_dir_all(&raw_dir).unwrap();

    for (filename, json) in router_api.to_schema_files().unwrap() {
        let path = raw_dir.join(filename);

        write(&path, json + "\n").unwrap();
        println!("Exported {}", path.to_str().unwrap());
    }
    // TODO: export the other ExecuteMsg responses like this
    export_schema(&schema_for!(lb_router::CreateLbPairResponse), &raw_dir);

    // ---

    let mut out_dir = env::current_dir().unwrap();

    out_dir.push("schema/lb_token");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    let token_api = generate_api! {
        instantiate: lb_token::InstantiateMsg,
        execute: lb_token::ExecuteMsg,
        // TODO: need to implement the QueryResponses trait on QueryMsg to tell cosmwasm_schema
        // which response type is associated with which query
        // query: lb_token::QueryMsg,
    }
    .render();

    let path = out_dir.join(concat!("lb_token", ".json"));

    let json = token_api.to_string().unwrap();
    write(&path, json + "\n").unwrap();
    println!("Exported the full API as {}", path.to_str().unwrap());

    let raw_dir = out_dir.join("raw");
    create_dir_all(&raw_dir).unwrap();

    for (filename, json) in token_api.to_schema_files().unwrap() {
        let path = raw_dir.join(filename);

        write(&path, json + "\n").unwrap();
        println!("Exported {}", path.to_str().unwrap());
    }
}
