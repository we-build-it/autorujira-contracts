use common::staking_provider::StakingProvider;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProtocolConfig {
    pub protocol: String,               // Protocol identifier (e.g., "AUTO", "NAMI")
    pub provider: StakingProvider,      // Associated staking provider (e.g., DAO_DAO)
    pub fee_percentage: Decimal,        // Fee percentage (e.g., "0.01" for 1%)
    pub fee_address: String,            // Address where the fee is sent
    pub claim_contract_address: String, // Address of the claim contract
    pub stake_contract_address: String, // Address of the stake contract
    pub reward_denom: String,           // Denomination of the reward token (e.g., "ukuji")
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigMsg {
    pub owner: Option<Addr>,
    pub max_parallel_claims: Option<u8>,
    pub protocol_configs: Vec<ProtocolConfig>, // List of protocol configurations
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<Addr>,
        max_parallel_claims: Option<u8>,
        protocol_configs: Vec<ProtocolConfig>,
    },
    ClaimAndStake {
        users_protocols: Vec<(String, Vec<String>)>, // List of users and their respective protocols
    },
    Subscribe {
        protocols: Vec<String>,
    },
    Unsubscribe {
        protocols: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the current contract configuration
    #[returns(ConfigResponse)]
    Config {},

    /// Returns the list of all subscriptions (address, [protocols])
    #[returns(GetSubscriptionsResponse)]
    GetSubscriptions {},

    /// Returns the list of protocols a specific address is subscribed to
    #[returns(GetSubscribedProtocolsResponse)]
    GetSubscribedProtocols { user_address: String },
}

/// Response structure for the config query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Option<Addr>,
    pub max_parallel_claims: Option<u8>,
    pub protocol_configs: Vec<ProtocolConfig>,
}

/// Response structure for the GetSubscriptions query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetSubscriptionsResponse {
    pub subscriptions: Vec<(String, Vec<String>)>, // List of user addresses and their protocols
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProtocolSubscriptionData {
    pub protocol: String,
    pub last_autoclaim: Option<u64>, // El timestamp de la última autoclaim, o None si no se ha ejecutado
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetSubscribedProtocolsResponse {
    pub protocols: Vec<ProtocolSubscriptionData>, // List of protocols with the last autoclaim timestamp for a specific user
}
