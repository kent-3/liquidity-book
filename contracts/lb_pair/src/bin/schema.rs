use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
// TODO: maybe we do have a problem...
use lb_interfaces::lb_pair::{
    ActiveIdResponse, AllBinsResponse, BinResponse, BinStepResponse, BinsResponse, ExecuteMsg,
    FactoryResponse, IdFromPriceResponse, InstantiateMsg, InvokeMsg, LbTokenResponse,
    LbTokenSupplyResponse, MintResponse, NextNonEmptyBinResponse, OracleParametersResponse,
    OracleSampleAtResponse, PriceFromIdResponse, ProtocolFeesResponse, QueryMsg, ReservesResponse,
    StaticFeeParametersResponse, SwapInResponse, SwapOutResponse, TokenXResponse, TokenYResponse,
    VariableFeeParametersResponse,
};
use std::{env, fs::create_dir_all, path::PathBuf};

fn main() {
    // Get the directory of the current crate
    let mut out_dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
    out_dir.push("schema");

    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(InvokeMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);

    export_schema(&schema_for!(LbTokenResponse), &out_dir);
    export_schema(&schema_for!(FactoryResponse), &out_dir);
    export_schema(&schema_for!(TokenXResponse), &out_dir);
    export_schema(&schema_for!(TokenYResponse), &out_dir);
    export_schema(&schema_for!(BinStepResponse), &out_dir);
    export_schema(&schema_for!(ReservesResponse), &out_dir);
    export_schema(&schema_for!(ActiveIdResponse), &out_dir);
    export_schema(&schema_for!(BinResponse), &out_dir);
    export_schema(&schema_for!(BinsResponse), &out_dir);
    export_schema(&schema_for!(AllBinsResponse), &out_dir);
    export_schema(&schema_for!(NextNonEmptyBinResponse), &out_dir);
    export_schema(&schema_for!(ProtocolFeesResponse), &out_dir);
    export_schema(&schema_for!(StaticFeeParametersResponse), &out_dir);
    export_schema(&schema_for!(VariableFeeParametersResponse), &out_dir);
    export_schema(&schema_for!(OracleParametersResponse), &out_dir);
    export_schema(&schema_for!(OracleSampleAtResponse), &out_dir);
    export_schema(&schema_for!(PriceFromIdResponse), &out_dir);
    export_schema(&schema_for!(IdFromPriceResponse), &out_dir);
    export_schema(&schema_for!(SwapInResponse), &out_dir);
    export_schema(&schema_for!(SwapOutResponse), &out_dir);
    export_schema(&schema_for!(LbTokenSupplyResponse), &out_dir);
    export_schema(&schema_for!(MintResponse), &out_dir);
}
