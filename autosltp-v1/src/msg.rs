use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Message used for the initial contract configuration during instantiation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: Addr,             // Owner address, mandatory at instantiation
}

/// Enum for defining the available contract execution messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    PlaceOrder { },
}

/// Enum for defining the available contract queries
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, QueryResponses)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the current contract configuration
    #[returns(ConfigResponse)]
    Config {},
}

/// Response structure for the config query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
}