use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum StakingProvider {
    DAO_DAO,
    CW_REWARDS,
}

impl std::str::FromStr for StakingProvider {
    type Err = ();

    fn from_str(input: &str) -> Result<StakingProvider, Self::Err> {
        match input {
            "CW_REWARDS" => Ok(StakingProvider::CW_REWARDS),
            "DAO_DAO" => Ok(StakingProvider::DAO_DAO),
            _ => Err(()),
        }
    }
}
