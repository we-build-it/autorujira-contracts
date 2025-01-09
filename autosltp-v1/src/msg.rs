use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Decimal, Uint128};
use rujira_rs::fin::{Price, Side};
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
    PlaceOrder { 
        fin_contract_address: Addr,
        side: Side,
        price: Price,
        amount: Uint128,
        price_sl: Option<Decimal>,
        price_tp: Option<Decimal>,
    },
    Protect { 
        fin_contract_address: Addr,
        side: Side,
        price: Price,
    },
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