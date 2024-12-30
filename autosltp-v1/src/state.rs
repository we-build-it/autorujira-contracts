use cosmwasm_std::{Addr};
use cw_storage_plus::{Item};
use serde::{Deserialize, Serialize};

/// Stores general AutoSLTP configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub owner: Addr, // Owner is now part of the overall configuration
}
pub const CONFIG: Item<Config> = Item::new("config");
