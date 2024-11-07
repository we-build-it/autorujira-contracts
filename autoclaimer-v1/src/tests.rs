#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Decimal};
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

    fn contract_autoclaimer() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query).with_reply(reply);
        Box::new(contract)
    }

    fn setup() -> (App, Contracts) {
        let mut app = AppBuilder::default().build(|_router, _api, _storage| {});

        let autoclaimer_code_id = app.store_code(contract_autoclaimer());
        let owner = Addr::unchecked("owner");
        let fee_address1 = Addr::unchecked("feeaddress1");
        let claim_contract1 = Addr::unchecked("claimcontract1");
        let stake_contract1 = Addr::unchecked("stakecontract1");
        let fee_address2 = Addr::unchecked("feeaddress2");
        let claim_contract2 = Addr::unchecked("claimcontract2");
        let stake_contract2 = Addr::unchecked("stakecontract2");

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
        let (app, contracts) = setup();
        let owner = Addr::unchecked("owner");

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
