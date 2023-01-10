use cosmwasm_std::{Coin, Addr};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub address: Addr,
    pub bid: Coin,
}

pub const STATE: Map<String, State> = Map::new("state");
pub const OWNER: Item<Addr> = Item::new("owner");
