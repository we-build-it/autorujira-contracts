use crate::{common_functions::build_authz_msg, staking_provider::StakingProvider};
use cosmwasm_std::{Addr, CosmosMsg, Env, StdResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClaimMsgDAODAO {
    pub claim: ClaimParamsDAODAO,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClaimParamsDAODAO {
    pub id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClaimMsgCwRewards {
    pub claim_rewards: ClaimParamsCwRewards,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClaimParamsCwRewards {}

/// Constructs an Authz message to claim rewards depending on the provider.
///
/// # Arguments
///
/// * `env` - The environment information.
/// * `user` - The address of the user who will claim the rewards.
/// * `provider` - The claim provider (DAO_DAO, CW_REWARDS).
/// * `claim_contract_address` - The address of the claim contract.
/// * `claim_id` - The ID of the claim.
///
/// # Returns
///
/// * `StdResult<CosmosMsg>` - The constructed Authz claim message.
pub fn build_claim_msg(
    env: Env,
    user: Addr,
    provider: StakingProvider,
    claim_contract_address: Addr,
    claim_id: u64,
) -> StdResult<CosmosMsg> {
    match provider {
        StakingProvider::DAO_DAO => {
            let claim_msg = ClaimMsgDAODAO {
                claim: ClaimParamsDAODAO { id: claim_id },
            };
            let claim_msg_str = serde_json::to_string(&claim_msg)
                .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;

            // Build the actual message, using a common function or direct construction
            build_authz_msg(env, user, claim_contract_address, claim_msg_str, vec![])
        }
        StakingProvider::CW_REWARDS => {
            let claim_msg = ClaimMsgCwRewards {
                claim_rewards: ClaimParamsCwRewards {},
            };
            let claim_msg_str = serde_json::to_string(&claim_msg)
                .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;

            // Build the actual message, using a common function or direct construction
            build_authz_msg(env, user, claim_contract_address, claim_msg_str, vec![])
        }
    }
}