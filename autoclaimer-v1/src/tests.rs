// src/tests.rs

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query, reply};
    use crate::msg::{
        ConfigResponse, ExecuteMsg, GetSubscribedProtocolsResponse, InstantiateMsg, ProtocolConfig,
        ProtocolStrategy, QueryMsg, UpdateConfigMsg,
    };
    use common::staking_provider::StakingProvider;
    use cosmwasm_std::{
        Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env, MessageInfo,
        Response, StdError, Uint128,
    };
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    // Import the mock structures and functions
    use crate::mocks::mock_functions::{ClaimMsg, MockClaimExecuteMsg, MockFINExecuteMsg, MockStakeExecuteMsg};

    struct Contracts {
        pub autoclaimer: Addr,
        pub claim_contract_success: Addr,
        pub fin_contract_addr: Addr,
    }

    fn contract_autoclaimer() -> Box<dyn Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    fn mock_claim_contract_success() -> Box<dyn Contract<Empty>> {
        let exec_fn = |_deps: DepsMut<Empty>,
                       _env: Env,
                       _info: MessageInfo,
                       msg: MockClaimExecuteMsg|
         -> Result<Response<Empty>, StdError> {
            match msg {
                MockClaimExecuteMsg::Claim(claim_msg) => {
                    // Simulate sending tokens to the user
                    Ok(Response::new().add_message(CosmosMsg::Bank(BankMsg::Send {
                        to_address: claim_msg.user_address.clone(),
                        amount: vec![Coin {
                            denom: "token1".to_string(), // Must match reward_denom
                            amount: Uint128::new(1000),  // Simulated amount
                        }],
                    })))
                }
            }
        };

        let instantiate_fn = |_deps: DepsMut<Empty>,
                              _env: Env,
                              _info: MessageInfo,
                              _msg: Empty|
         -> Result<Response<Empty>, StdError> { Ok(Response::new()) };

        let query_fn = |_deps: Deps<Empty>, _env: Env, _msg: Empty| -> Result<Binary, StdError> {
            Ok(Binary::default())
        };

        let contract = ContractWrapper::new_with_empty(exec_fn, instantiate_fn, query_fn);
        Box::new(contract)
    }

    fn mock_claim_contract_failure() -> Box<dyn Contract<Empty>> {
        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
        pub enum MockFailExecuteMsg {
            Claim(ClaimMsg),
        }

        let exec_fn = |_deps: DepsMut<Empty>,
                       _env: Env,
                       _info: MessageInfo,
                       msg: MockFailExecuteMsg|
         -> Result<Response<Empty>, StdError> {
            match msg {
                MockFailExecuteMsg::Claim(_claim_msg) => {
                    Err(StdError::generic_err("Mock claim contract failure"))
                }
            }
        };

        let instantiate_fn = |_deps: DepsMut<Empty>,
                              _env: Env,
                              _info: MessageInfo,
                              _msg: Empty|
         -> Result<Response<Empty>, StdError> { Ok(Response::new()) };

        let query_fn = |_deps: Deps<Empty>, _env: Env, _msg: Empty| -> Result<Binary, StdError> {
            Ok(Binary::default())
        };

        let contract = ContractWrapper::new_with_empty(exec_fn, instantiate_fn, query_fn);

        Box::new(contract)
    }

    fn mock_stake_contract() -> Box<dyn Contract<Empty>> {
        let exec_fn = |_deps: DepsMut<Empty>,
                       _env: Env,
                       _info: MessageInfo,
                       msg: MockStakeExecuteMsg|
         -> Result<Response<Empty>, StdError> {
            match msg {
                MockStakeExecuteMsg::Stake(stake_msg) => {
                    // For testing, you can verify that the amount is correct
                    assert!(
                        stake_msg.amount > Uint128::zero(),
                        "Stake amount should be greater than zero"
                    );
                    Ok(Response::new())
                }
            }
        };

        let instantiate_fn = |_deps: DepsMut<Empty>,
                              _env: Env,
                              _info: MessageInfo,
                              _msg: Empty|
         -> Result<Response<Empty>, StdError> { Ok(Response::new()) };

        let query_fn = |_deps: Deps<Empty>, _env: Env, _msg: Empty| -> Result<Binary, StdError> {
            Ok(Binary::default())
        };

        let contract = ContractWrapper::new_with_empty(exec_fn, instantiate_fn, query_fn);

        Box::new(contract)
    }

    fn mock_fin_contract() -> Box<dyn Contract<Empty>> {
        let exec_fn = |_deps: DepsMut<Empty>,
                       _env: Env,
                       _info: MessageInfo,
                       msg: MockFINExecuteMsg|
         -> Result<Response<Empty>, StdError> {
            match msg {
                MockFINExecuteMsg::WithdrawOrders {} => {
                    // Simulate success
                    Ok(Response::new())
                }
            }
        };

        let instantiate_fn = |_deps: DepsMut<Empty>,
                              _env: Env,
                              _info: MessageInfo,
                              _msg: Empty|
         -> Result<Response<Empty>, StdError> { Ok(Response::new()) };

        let query_fn = |_deps: Deps<Empty>, _env: Env, _msg: Empty| -> Result<Binary, StdError> {
            Ok(Binary::default())
        };

        let contract = ContractWrapper::new_with_empty(exec_fn, instantiate_fn, query_fn);

        Box::new(contract)
    }

    fn setup() -> (App, Contracts) {
        let mut app = AppBuilder::default().build(|_router, _api, _storage| {});

        let autoclaimer_code_id = app.store_code(contract_autoclaimer());

        // Store mock claim, stake, and FIN contracts
        let claim_contract_success_code_id = app.store_code(mock_claim_contract_success());
        let claim_contract_failure_code_id = app.store_code(mock_claim_contract_failure());
        let stake_contract_code_id = app.store_code(mock_stake_contract());
        let fin_contract_code_id = app.store_code(mock_fin_contract());

        let owner = Addr::unchecked("owner");

        // Instantiate the mock claim contracts
        let claim_contract_success_addr = app
            .instantiate_contract(
                claim_contract_success_code_id,
                owner.clone(),
                &Empty {},
                &[],
                "Mock Claim Contract Success",
                None,
            )
            .unwrap();

        let claim_contract_failure_addr = app
            .instantiate_contract(
                claim_contract_failure_code_id,
                owner.clone(),
                &Empty {},
                &[],
                "Mock Claim Contract Failure",
                None,
            )
            .unwrap();

        // Instantiate the mock stake contract
        let stake_contract_addr = app
            .instantiate_contract(
                stake_contract_code_id,
                owner.clone(),
                &Empty {},
                &[],
                "Mock Stake Contract",
                None,
            )
            .unwrap();

        // Instantiate the mock FIN contract
        let fin_contract_addr = app
            .instantiate_contract(
                fin_contract_code_id,
                owner.clone(),
                &Empty {},
                &[],
                "Mock FIN Contract",
                None,
            )
            .unwrap();

        // Use these addresses in the InstantiateMsg
        let instantiate_msg = InstantiateMsg {
            owner: owner.clone(),
            max_parallel_claims: 5,
            protocol_configs: vec![
                ProtocolConfig {
                    protocol: "protocol1".to_string(),
                    fee_percentage: Decimal::percent(1),
                    fee_address: "feeaddress1".to_string(),
                    strategy: ProtocolStrategy::ClaimAndStakeDaoDaoCwRewards {
                        provider: StakingProvider::CW_REWARDS,
                        claim_contract_address: claim_contract_success_addr.to_string(),
                        stake_contract_address: stake_contract_addr.to_string(),
                        reward_denom: "token1".to_string(),
                    },
                },
                ProtocolConfig {
                    protocol: "protocol2".to_string(),
                    fee_percentage: Decimal::percent(1),
                    fee_address: "feeaddress2".to_string(),
                    strategy: ProtocolStrategy::ClaimAndStakeDaoDaoCwRewards {
                        provider: StakingProvider::CW_REWARDS,
                        claim_contract_address: claim_contract_failure_addr.to_string(),
                        stake_contract_address: stake_contract_addr.to_string(),
                        reward_denom: "token2".to_string(),
                    },
                },
                ProtocolConfig {
                    protocol: "FIN".to_string(),
                    fee_percentage: Decimal::zero(), // Assuming no fee
                    fee_address: "".to_string(),
                    strategy: ProtocolStrategy::ClaimOnlyFIN {
                        supported_markets: vec![fin_contract_addr.to_string()],
                    },
                },
            ],
        };

        let autoclaimer_addr = app
            .instantiate_contract(
                autoclaimer_code_id,
                owner.clone(),
                &instantiate_msg,
                &[],
                "Autoclaimer",
                None,
            )
            .unwrap();

        (
            app,
            Contracts {
                autoclaimer: autoclaimer_addr,
                claim_contract_success: claim_contract_success_addr,
                fin_contract_addr,
            },
        )
    }

    #[test]
    fn test_claim_only_fin() {
        let (mut app, contracts) = setup();

        let owner = Addr::unchecked("owner");
        let user = Addr::unchecked("user1");

        // Subscribe the user to the FIN protocol
        let subscribe_msg = ExecuteMsg::Subscribe {
            protocols: vec!["FIN".to_string()],
        };

        app.execute_contract(
            user.clone(),
            contracts.autoclaimer.clone(),
            &subscribe_msg,
            &[],
        )
        .unwrap();

        // Prepare the list of user contracts (user and fin_contract_address)
        let users_contracts = vec![(user.to_string(), contracts.fin_contract_addr.to_string())];

        // Execute ClaimOnly as owner
        let claim_only_msg = ExecuteMsg::ClaimOnly {
            protocol: "FIN".to_string(),
            users_contracts,
        };

        let res = app.execute_contract(
            owner.clone(),
            contracts.autoclaimer.clone(),
            &claim_only_msg,
            &[],
        );

        assert!(res.is_ok(), "Execution failed: {:?}", res.unwrap_err());

        let res = res.unwrap();

        // Check that the events contain the expected messages
        let mut claim_ok_found = false;

        for event in res.events {
            if event.ty == "wasm-autorujira.autoclaimer" {
                println!("Event: {:?}", event);
                let mut action = None;
                let mut result = None;

                for attr in &event.attributes {
                    match attr.key.as_str() {
                        "action" => action = Some(attr.value.clone()),
                        "result" => result = Some(attr.value.clone()),
                        _ => {}
                    }
                }

                if action == Some("claim".to_string()) && result == Some("ok".to_string()) {
                    claim_ok_found = true;
                }
            }
        }

        assert!(claim_ok_found, "claim ok event for FIN not found");

        // Check that last_autoclaim is updated for FIN
        let res: GetSubscribedProtocolsResponse = app
            .wrap()
            .query_wasm_smart(
                contracts.autoclaimer.clone(),
                &QueryMsg::GetSubscribedProtocols {
                    user_address: user.to_string(),
                },
            )
            .unwrap();

        for protocol_data in res.protocols {
            if protocol_data.protocol == "FIN" {
                assert!(
                    protocol_data.last_autoclaim.is_some(),
                    "last_autoclaim should be updated for FIN"
                );
            }
        }
    }

    #[test]
    fn test_unauthorized_claim_only_fin() {
        let (mut app, contracts) = setup();
        let user = Addr::unchecked("user1");

        // Subscribe the user to the FIN protocol
        let subscribe_msg = ExecuteMsg::Subscribe {
            protocols: vec!["FIN".to_string()],
        };
        app.execute_contract(
            user.clone(),
            contracts.autoclaimer.clone(),
            &subscribe_msg,
            &[],
        )
        .unwrap();

        // Prepare the list of user contracts (user and fin_contract_address)
        let users_contracts = vec![(user.to_string(), contracts.fin_contract_addr.to_string())];

        // Attempt to execute ClaimOnly as user (not owner)
        let claim_only_msg = ExecuteMsg::ClaimOnly {
            protocol: "FIN".to_string(),
            users_contracts,
        };

        let err = app
            .execute_contract(
                user.clone(),
                contracts.autoclaimer.clone(),
                &claim_only_msg,
                &[],
            )
            .unwrap_err();

        println!("Error: {:?}", err);
        assert!(err
            .root_cause()
            .to_string()
            .contains("You have no permissions to execute this function"));
    }

    #[test]
    fn test_claim_and_stake_with_failures() {
        let (mut app, contracts) = setup();

        let owner = Addr::unchecked("owner");
        let user = Addr::unchecked("user1");

        use cw_multi_test::BankSudo;

        // Ensure the claim contract has enough balance to send tokens
        app.sudo(cw_multi_test::SudoMsg::Bank(BankSudo::Mint {
            to_address: contracts.claim_contract_success.to_string(),
            amount: vec![Coin {
                denom: "token1".to_string(),
                amount: Uint128::new(1000),
            }],
        }))
        .unwrap();

        // Ensure the autoclaimer contract has enough balance to send tokens
        app.sudo(cw_multi_test::SudoMsg::Bank(BankSudo::Mint {
            to_address: contracts.autoclaimer.to_string(),
            amount: vec![Coin {
                denom: "token1".to_string(),
                amount: Uint128::new(1000),
            }],
        }))
        .unwrap();

        // Subscribe the user to both protocols
        let subscribe_msg = ExecuteMsg::Subscribe {
            protocols: vec!["protocol1".to_string(), "protocol2".to_string()],
        };

        app.execute_contract(
            user.clone(),
            contracts.autoclaimer.clone(),
            &subscribe_msg,
            &[],
        )
        .unwrap();

        // Execute ClaimAndStake as owner
        let claim_and_stake_msg = ExecuteMsg::ClaimAndStake {
            users_protocols: vec![(
                user.to_string(),
                vec!["protocol1".to_string(), "protocol2".to_string()],
            )],
        };

        let res = app.execute_contract(
            owner.clone(),
            contracts.autoclaimer.clone(),
            &claim_and_stake_msg,
            &[],
        );

        assert!(res.is_ok(), "Execution failed: {:?}", res.unwrap_err());

        let res = res.unwrap();

        // Check that the events contain the expected messages
        let mut claim_failed_found = false;
        let mut claim_ok_found = false;
        let mut stake_ok_found = false;
        let mut charge_fee_ok_found = false;

        for event in res.events {
            if event.ty == "wasm-autorujira.autoclaimer" {
                println!("Event: {:?}", event);
                let mut action = None;
                let mut protocol = None;
                let mut result = None;
                let mut msg_id = None;

                for attr in &event.attributes {
                    match attr.key.as_str() {
                        "action" => action = Some(attr.value.clone()),
                        "protocol" => protocol = Some(attr.value.clone()),
                        "result" => result = Some(attr.value.clone()),
                        "msg_id" => msg_id = Some(attr.value.clone()),
                        _ => {}
                    }
                }

                if action == Some("claim".to_string())
                    && protocol == Some("protocol2".to_string())
                    && result == Some("failed".to_string())
                {
                    claim_failed_found = true;
                }

                if action == Some("claim".to_string())
                    && protocol == Some("protocol1".to_string())
                    && result == Some("ok".to_string())
                {
                    claim_ok_found = true;
                }

                if action == Some("charge_fee".to_string())
                    && result == Some("ok".to_string())
                    && msg_id == Some("3000".to_string())
                {
                    charge_fee_ok_found = true;
                }

                if action == Some("stake".to_string())
                    && result == Some("ok".to_string())
                    && msg_id == Some("2000".to_string())
                {
                    stake_ok_found = true;
                }
            }
        }

        assert!(
            claim_failed_found,
            "claim failed event for protocol2 not found"
        );
        assert!(claim_ok_found, "claim ok event for protocol1 not found");
        assert!(stake_ok_found, "stake ok event not found");
        assert!(charge_fee_ok_found, "charge fee ok event not found");

        // Optionally, check that last_autoclaim is updated for protocol1 but not for protocol2
        let res: GetSubscribedProtocolsResponse = app
            .wrap()
            .query_wasm_smart(
                contracts.autoclaimer.clone(),
                &QueryMsg::GetSubscribedProtocols {
                    user_address: user.to_string(),
                },
            )
            .unwrap();

        for protocol_data in res.protocols {
            if protocol_data.protocol == "protocol1" {
                assert!(
                    protocol_data.last_autoclaim.is_some(),
                    "last_autoclaim should be updated for protocol1"
                );
            } else if protocol_data.protocol == "protocol2" {
                assert!(
                    protocol_data.last_autoclaim.is_none(),
                    "last_autoclaim should not be updated for protocol2"
                );
            }
        }
    }

    #[test]
    fn test_instantiate_and_query_config() {
        let (app, contracts) = setup();
        let owner = Addr::unchecked("owner");

        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contracts.autoclaimer.clone(), &QueryMsg::Config {})
            .unwrap();

        assert_eq!(config.owner, owner);
        assert_eq!(config.max_parallel_claims, 5);
        assert_eq!(config.protocol_configs.len(), 3);
        assert_eq!(config.protocol_configs[0].protocol, "FIN");
        assert_eq!(config.protocol_configs[1].protocol, "protocol1");
        assert_eq!(config.protocol_configs[2].protocol, "protocol2");
    }

    #[test]
    fn test_subscribe_and_query_subscriptions() {
        let (mut app, contracts) = setup();
        let user = Addr::unchecked("user1");
        let subscribe_msg = ExecuteMsg::Subscribe {
            protocols: vec!["protocol1".to_string(), "protocol2".to_string()],
        };

        app.execute_contract(
            user.clone(),
            contracts.autoclaimer.clone(),
            &subscribe_msg,
            &[],
        )
        .unwrap();

        let res: GetSubscribedProtocolsResponse = app
            .wrap()
            .query_wasm_smart(
                contracts.autoclaimer.clone(),
                &QueryMsg::GetSubscribedProtocols {
                    user_address: user.to_string(),
                },
            )
            .unwrap();
        assert_eq!(res.protocols.len(), 2);
        assert_eq!(res.protocols[0].protocol, "protocol1");
        assert_eq!(res.protocols[1].protocol, "protocol2");
    }

    #[test]
    fn test_unsubscribe() {
        let (mut app, contracts) = setup();
        let user = Addr::unchecked("user1");
        let subscribe_msg = ExecuteMsg::Subscribe {
            protocols: vec!["protocol1".to_string(), "protocol2".to_string()],
        };
        app.execute_contract(
            user.clone(),
            contracts.autoclaimer.clone(),
            &subscribe_msg,
            &[],
        )
        .unwrap();

        let unsubscribe_msg = ExecuteMsg::Unsubscribe {
            protocols: vec!["protocol1".to_string()],
        };
        app.execute_contract(
            user.clone(),
            contracts.autoclaimer.clone(),
            &unsubscribe_msg,
            &[],
        )
        .unwrap();

        let res: GetSubscribedProtocolsResponse = app
            .wrap()
            .query_wasm_smart(
                contracts.autoclaimer.clone(),
                &QueryMsg::GetSubscribedProtocols {
                    user_address: user.to_string(),
                },
            )
            .unwrap();
        assert_eq!(res.protocols.len(), 1);
        assert_eq!(res.protocols[0].protocol, "protocol2");
    }

    #[test]
    fn test_unauthorized_claim_and_stake() {
        let (mut app, contracts) = setup();
        let user = Addr::unchecked("user1");
        let subscribe_msg = ExecuteMsg::Subscribe {
            protocols: vec!["protocol1".to_string()],
        };
        app.execute_contract(
            user.clone(),
            contracts.autoclaimer.clone(),
            &subscribe_msg,
            &[],
        )
        .unwrap();

        let claim_and_stake_msg = ExecuteMsg::ClaimAndStake {
            users_protocols: vec![(user.to_string(), vec!["protocol1".to_string()])],
        };
        let err = app
            .execute_contract(
                user.clone(),
                contracts.autoclaimer.clone(),
                &claim_and_stake_msg,
                &[],
            )
            .unwrap_err();

        println!("Error: {:?}", err);
        assert!(err
            .root_cause()
            .to_string()
            .contains("You have no permissions to execute this function"));
    }

    #[test]
    fn test_update_config() {
        let (mut app, contracts) = setup();
        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfigMsg {
                owner: Some(Addr::unchecked("new_owner")),
                max_parallel_claims: Some(10),
                protocol_configs: None,
            },
        };
        app.execute_contract(
            Addr::unchecked("owner"),
            contracts.autoclaimer.clone(),
            &update_msg,
            &[],
        )
        .unwrap();

        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contracts.autoclaimer.clone(), &QueryMsg::Config {})
            .unwrap();
        assert_eq!(config.owner, Addr::unchecked("new_owner"));
        assert_eq!(config.max_parallel_claims, 10);
    }
}
