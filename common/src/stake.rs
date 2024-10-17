use cosmwasm_std::{Addr, Coin, CosmosMsg, Env, StdResult};
use serde::{Deserialize, Serialize};
use crate::{common_functions::build_authz_msg, staking_provider::StakingProvider};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StakeContractExecuteMsg {
    Stake {},
}

/// Constructs an Authz message to stake tokens depending on the provider.
///
/// # Arguments
///
/// * `env` - The environment information.
/// * `user` - The address of the user who will stake the tokens.
/// * `provider` - The staking provider (DAO_DAO, CW_REWARDS).
/// * `stake_contract_address` - The address of the stake contract.
/// * `amount` - The amount to stake.
/// * `denom` - The denomination of the token to stake.
///
/// # Returns
///
/// * `StdResult<CosmosMsg>` - The constructed Authz stake message.
pub fn build_stake_msg(
    env: Env,
    user: Addr,
    provider: StakingProvider,
    stake_contract_address: Addr,
    amount: u128,
    denom: String,
) -> StdResult<CosmosMsg> {
    match provider {
        StakingProvider::DAO_DAO | StakingProvider::CW_REWARDS => {
            let stake_msg = StakeContractExecuteMsg::Stake {};
            let stake_msg_str = serde_json::to_string(&stake_msg)
                .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;

            let funds = vec![Coin {
                denom,
                amount: amount.into(),
            }];

            // Build the actual message, using a common function or direct construction
            build_authz_msg(env, user, stake_contract_address, stake_msg_str, funds)
        }
    }
}