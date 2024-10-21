use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

use crate::msg::ProtocolConfig;

/// Stores general AutoClaimer configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Config {
    pub owner: Addr, // Owner is now part of the overall configuration
    pub max_parallel_claims: u8,
}

pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the configuration for each protocol, accessible by its name (String).
pub const PROTOCOL_CONFIG: Map<&str, ProtocolConfig> = Map::new("protocol_config");

/// Stores user subscriptions, accessible by the user address.
pub const SUBSCRIPTIONS: Map<&Addr, Vec<String>> = Map::new("subscriptions");

/// Stores operational data like last_autoclaim and potentially other execution metadata
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExecutionData {
    pub last_autoclaim: Timestamp,
}

pub const USER_EXECUTION_DATA: Map<(Addr, String), ExecutionData> = Map::new("user_execution_data");

/// Stores user, protocol, and balance_before for each reply_id.
pub const PENDING_USER_PROTOCOL: Map<u64, (Addr, String, Uint128)> =
    Map::new("pending_user_protocol");
