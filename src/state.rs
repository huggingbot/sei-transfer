use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub cw20_addr: Addr,
    pub cw20_decimals: u32,
    pub fee_basis_point: Uint64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const WITHDRAWABLE: Map<Addr, Uint128> = Map::new("withdrawable");
