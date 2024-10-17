use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

use crate::msg::ProtocolConfig;

//  Contract owner for securing fn execution only by the owner
pub const OWNER: Item<Addr> = Item::new("admin");

// Stores the configuration for each protocol, accessible by its name (String).
pub const CONFIG: Map<&str, ProtocolConfig> = Map::new("config");

// Max claims executed in ||
pub const MAX_PARALLEL_CLAIMS_STORAGE: Item<u8> = Item::new("max_parallel_claims");

// Stores user subscriptions, accessible by the user address.
pub const SUBSCRIPTIONS: Map<&Addr, Vec<String>> = Map::new("subscriptions");

// This stores operational data like last_autoclaim and potentially other execution metadata
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExecutionData {
    pub last_autoclaim: Timestamp, // Timestamp of the last autoclaim execution
                                   // Future operational data can be added here (e.g., number of claims, total claimed, etc.)
}
pub const USER_EXECUTION_DATA: Map<(Addr, String), ExecutionData> = Map::new("user_execution_data");

// Stores user, protocol, and balance_before for each reply_id.
pub const PENDING_USER_PROTOCOL: Map<u64, (Addr, String, Uint128)> =
    Map::new("pending_user_protocol");
