use common::staking_provider::StakingProvider;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Common configuration for all protocols
// Define the old ProtocolConfig struct matching the old data structure
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OldProtocolConfig {
    pub provider: StakingProvider,
    pub claim_contract_address: String,
    pub stake_contract_address: String,
    pub reward_denom: String,
    pub fee_percentage: Decimal,
    pub fee_address: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProtocolConfig {
    pub protocol: String,        // Protocol identifier (e.g., "AUTO", "MNTA", "FIN")
    pub fee_percentage: Decimal, // Fee percentage (e.g., "0.01" for 1%)
    pub fee_address: String,     // Address where the fee is sent
    pub strategy: ProtocolStrategy, // Specific strategy for the protocol
}

/// Enum for defining the strategy of a protocol
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum ProtocolStrategy {
    /// Strategy for claim and stake (e.g., AUTO, MNTA)
    ClaimAndStakeDaoDaoCwRewards {
        provider: StakingProvider, // Associated staking provider (e.g., CW_REWARDS)
        claim_contract_address: String, // Address of the claim contract
        stake_contract_address: String, // Address of the stake contract
        reward_denom: String,      // Denomination of the reward token (e.g., "ukuji")
    },
    /// Strategy for claim only (e.g., FIN)
    ClaimOnlyFIN {
        supported_markets: Vec<String>, // List of supported market contract addresses
    },
}

impl ProtocolStrategy {
    /// Convert the ProtocolStrategy into a string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ProtocolStrategy::ClaimAndStakeDaoDaoCwRewards { .. } => "ClaimAndStakeDaoDaoCwRewards",
            ProtocolStrategy::ClaimOnlyFIN { .. } => "ClaimOnlyFIN",
            // Agrega aquí otras estrategias según sea necesario
        }
    }
}
/// Message used for the initial contract configuration during instantiation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: Addr,             // Owner address, mandatory at instantiation
    pub max_parallel_claims: u8, // Maximum number of parallel claims
    pub protocol_configs: Vec<ProtocolConfig>, // List of protocol configurations
}

/// Message used for updating the contract configuration
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfigMsg {
    pub owner: Option<Addr>,                           // Optional owner update
    pub max_parallel_claims: Option<u8>,               // Optional max parallel claims update
    pub protocol_configs: Option<Vec<ProtocolConfig>>, // Optional protocol configuration update
}

/// Enum for defining the available contract execution messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        config: UpdateConfigMsg,
    },
    ClaimAndStake {
        users_protocols: Vec<(String, Vec<String>)>, // List of users and their respective protocols
    },
    ClaimOnly {
        protocol: String,
        users_contracts: Vec<(String, String)>, // (user_address, contract_address)
    },
    Subscribe {
        protocols: Vec<String>, // Protocols to subscribe to
    },
    Unsubscribe {
        protocols: Vec<String>, // Protocols to unsubscribe from
    },
}

/// Enum for defining the available contract queries
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
    pub owner: Addr,
    pub max_parallel_claims: u8,
    pub protocol_configs: Vec<ProtocolConfig>,
}

/// Response structure for the GetSubscriptions query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetSubscriptionsResponse {
    pub subscriptions: Vec<(String, Vec<String>)>, // List of user addresses and their protocols
}

/// Data structure to represent protocol subscription data
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProtocolSubscriptionData {
    pub protocol: String,
    pub last_autoclaim: Option<u64>, // Timestamp of the last autoclaim, or None if never executed
}

/// Response structure for the GetSubscribedProtocols query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetSubscribedProtocolsResponse {
    pub protocols: Vec<ProtocolSubscriptionData>, // List of protocols with the last autoclaim timestamp for a specific user
}
