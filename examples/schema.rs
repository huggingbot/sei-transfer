use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use sei_transfer::msg::{
    Cw20ContractResponse, Cw20DecimalsResponse, ExecuteMsg, GetFeeBasisPointResponse,
    InstantiateMsg, OwnerResponse, QueryMsg, ReceiveMsg, WithdrawableResponse,
};
use sei_transfer::state::Config;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ReceiveMsg), &out_dir);
    export_schema(&schema_for!(OwnerResponse), &out_dir);
    export_schema(&schema_for!(Cw20ContractResponse), &out_dir);
    export_schema(&schema_for!(Cw20DecimalsResponse), &out_dir);
    export_schema(&schema_for!(WithdrawableResponse), &out_dir);
    export_schema(&schema_for!(GetFeeBasisPointResponse), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
}
