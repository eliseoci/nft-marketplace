use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub native_denom: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw721Deposits {
    pub owner: String,
    pub collection: String,
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    pub collection: String,
    pub token_id: String,
    pub seller: String,
    pub price: Uint128,
    pub cw20_contract: Option<String>,
}

pub const CONFIG: Item<Config> = Item::new("config");
//contract, owner, token_id
pub const CW721_DEPOSITS: Map<(&str, &str, &str), Cw721Deposits> = Map::new("cw721deposits");

//key can be cw721_contract, token_id
pub const ASKS: Map<(&str, &str), Ask> = Map::new("asks");
