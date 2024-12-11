// src/mocks.rs

#[cfg(test)]
pub mod mock_functions {
    use crate::error::ContractError;
    use common::staking_provider::StakingProvider;
    use cosmwasm_std::{to_json_binary, Addr, BankMsg, Coin, CosmosMsg, Env, Uint128, WasmMsg};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    // Define ClaimMsg struct
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct ClaimMsg {
        pub user_address: String,
    }

    // Define StakeMsg struct
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct StakeMsg {
        pub amount: Uint128,
        pub denom: String,
    }

    // Define execute messages for mock claim contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub enum MockClaimExecuteMsg {
        Claim(ClaimMsg),
    }

    // Define execute messages for mock stake contract
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub enum MockStakeExecuteMsg {
        Stake(StakeMsg),
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub enum MockFINExecuteMsg {
        WithdrawOrders(),
    }

    pub fn build_claim_msg(
        _env: Env,
        user: Addr,
        _provider: StakingProvider,
        claim_contract_addr: Addr,
        _claim_id: u64,
    ) -> Result<CosmosMsg, ContractError> {
        let claim_msg = MockClaimExecuteMsg::Claim(ClaimMsg {
            user_address: user.to_string(),
        });
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: claim_contract_addr.to_string(),
            msg: to_json_binary(&claim_msg)?,
            funds: vec![],
        }))
    }

    pub fn build_stake_msg(
        _env: Env,
        _user: Addr,
        _provider: StakingProvider,
        stake_contract_addr: Addr,
        amount: u128,
        denom: String,
    ) -> Result<CosmosMsg, ContractError> {
        let stake_msg = MockStakeExecuteMsg::Stake(StakeMsg {
            amount: Uint128::from(amount),
            denom: denom.clone(),
        });
        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: stake_contract_addr.to_string(),
            msg: to_json_binary(&stake_msg)?,
            funds: vec![Coin {
                denom,
                amount: Uint128::from(amount),
            }],
        }))
    }

    pub fn build_send_msg(
        _env: Env,
        _user: Addr,
        to_address: Addr,
        amount: u128,
        denom: String,
    ) -> Result<CosmosMsg, ContractError> {
        Ok(CosmosMsg::Bank(BankMsg::Send {
            to_address: to_address.to_string(),
            amount: vec![cosmwasm_std::Coin {
                denom: denom,
                amount: amount.into(),
            }],
        }))
    }

    pub fn build_FIN_claim_msg(
        _env: Env,
        _user: Addr,
        contract_address: Addr,
    ) -> Result<CosmosMsg, ContractError> {
        let claim_msg = MockFINExecuteMsg::WithdrawOrders();

        Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_address.to_string(),
            msg: to_json_binary(&claim_msg)?,
            funds: vec![],
        }))
    }
}
