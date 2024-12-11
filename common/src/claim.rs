use crate::{
    common_functions::{build_authz_msg, AuthzMessageType},
    staking_provider::StakingProvider,
};
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
    // Process the claim message within each branch to avoid type mismatch
    let claim_msg_str = match provider {
        StakingProvider::DAO_DAO => {
            let claim_msg = ClaimMsgDAODAO {
                claim: ClaimParamsDAODAO { id: claim_id },
            };
            serde_json::to_string(&claim_msg)
                .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?
        }
        StakingProvider::CW_REWARDS => {
            let claim_msg = ClaimMsgCwRewards {
                claim_rewards: ClaimParamsCwRewards {},
            };
            serde_json::to_string(&claim_msg)
                .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?
        }
    };

    build_authz_msg(
        env,
        user,
        AuthzMessageType::ExecuteContract {
            contract_addr: claim_contract_address,
            msg_str: claim_msg_str,
            funds: vec![],
        },
    )
}

pub fn build_FIN_claim_msg(env: Env, user: Addr, contract_address: Addr) -> StdResult<CosmosMsg> {
    let claim_msg = serde_json::to_string(&serde_json::json!({ "withdraw_orders": {} }))
        .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;
    build_authz_msg(
        env,
        user,
        AuthzMessageType::ExecuteContract {
            contract_addr: contract_address,
            msg_str: claim_msg,
            funds: vec![],
        },
    )
}
