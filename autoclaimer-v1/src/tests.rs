#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MockApi;
    use cosmwasm_std::{Addr, Coin, Decimal};
    use cw_multi_test::{App, AppBuilder, ContractWrapper, Executor};

    use crate::contract::{execute, instantiate, query, reply};
    use crate::msg::{
        ConfigResponse, ExecuteMsg, GetSubscribedProtocolsResponse, InstantiateMsg, ProtocolConfig,
        QueryMsg, UpdateConfigMsg,
    };
    use common::staking_provider::StakingProvider;

    struct Contracts {
        pub autoclaimer: Addr,
    }

    // The contract still uses cosmwasm_std::Empty
    fn contract_autoclaimer() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    fn setup(balances: Vec<(Addr, Vec<Coin>)>) -> (App, Contracts) {
        // Create the API with the Bech32 prefix "kujira"
        let api = MockApi::default().with_prefix("kujira");

        // Create the App using AppBuilder and the custom API
        let mut app = AppBuilder::new_custom()
            .with_api(api.clone())
            .build(|_router, _api, _storage| {});

        // Initialize balances using BankSudo::Mint
        for (addr, coins) in balances {
            app.sudo(cw_multi_test::SudoMsg::Bank({
                cw_multi_test::BankSudo::Mint {
                    to_address: addr.to_string(),
                    amount: coins,
                }
            }))
            .unwrap();
        }

        let autoclaimer_code_id = app.store_code(contract_autoclaimer());

        // Create addresses using the API
        let owner = app.api().addr_make("owner");
        let fee_address1 = app.api().addr_make("feeaddress1");
        let claim_contract1 = app.api().addr_make("claimcontract1");
        let stake_contract1 = app.api().addr_make("stakecontract1");
        let fee_address2 = app.api().addr_make("feeaddress2");
        let claim_contract2 = app.api().addr_make("claimcontract2");
        let stake_contract2 = app.api().addr_make("stakecontract2");

        let instantiate_msg = InstantiateMsg {
            owner: owner.clone(),
            max_parallel_claims: 5,
            protocol_configs: vec![
                ProtocolConfig {
                    protocol: "protocol1".to_string(),
                    provider: StakingProvider::DAO_DAO,
                    fee_percentage: Decimal::zero(),
                    fee_address: fee_address1.to_string(),
                    claim_contract_address: claim_contract1.to_string(),
                    stake_contract_address: stake_contract1.to_string(),
                    reward_denom: "token1".to_string(),
                },
                ProtocolConfig {
                    protocol: "protocol2".to_string(),
                    provider: StakingProvider::CW_REWARDS,
                    fee_percentage: Decimal::percent(1),
                    fee_address: fee_address2.to_string(),
                    claim_contract_address: claim_contract2.to_string(),
                    stake_contract_address: stake_contract2.to_string(),
                    reward_denom: "token2".to_string(),
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
            },
        )
    }

    #[test]
    fn test_instantiate_and_query_config() {
        let (app, contracts) = setup(vec![]); // Empty initial balances
        let owner = app.api().addr_make("owner");

        // Query the configuration
        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contracts.autoclaimer.clone(), &QueryMsg::Config {})
            .unwrap();

        assert_eq!(config.owner, owner);
        assert_eq!(config.max_parallel_claims, 5);
        assert_eq!(config.protocol_configs.len(), 2);
        assert_eq!(config.protocol_configs[0].protocol, "protocol1");
        assert_eq!(config.protocol_configs[1].protocol, "protocol2");
    }

    #[test]
    fn test_subscribe_and_query_subscriptions() {
        let (mut app, contracts) = setup(vec![]);

        let user = app.api().addr_make("user1");
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
        let (mut app, contracts) = setup(vec![]); // Empty initial balances

        // The user subscribes to the protocols
        let user = app.api().addr_make("user1");
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

        // The user unsubscribes from protocol1
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

        // Query the user's subscriptions
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
        let (mut app, contracts) = setup(vec![]);
        let user = app.api().addr_make("user1");
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

        // An unauthorized user tries to execute ClaimAndStake
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

        // Print the actual error message
        println!("Error: {:?}", err);

        // Adjust the assertion based on the actual error message
        assert!(err
            .root_cause()
            .to_string()
            .contains("You have no permissions to execute this function"));
    }

    #[test]
    fn test_update_config() {
        let (mut app, contracts) = setup(vec![]); // Empty initial balances

        // The owner updates the configuration
        let update_msg = ExecuteMsg::UpdateConfig {
            config: UpdateConfigMsg {
                owner: Some(app.api().addr_make("new_owner")),
                max_parallel_claims: Some(10),
                protocol_configs: None,
            },
        };
        app.execute_contract(
            app.api().addr_make("owner"),
            contracts.autoclaimer.clone(),
            &update_msg,
            &[],
        )
        .unwrap();

        // Query the updated configuration
        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contracts.autoclaimer.clone(), &QueryMsg::Config {})
            .unwrap();
        assert_eq!(config.owner, app.api().addr_make("new_owner"));
        assert_eq!(config.max_parallel_claims, 10);
    }
}
