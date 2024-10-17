#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::Addr;

    use crate::contract::{execute, query_get_subscribed_protocols, query_get_subscriptions};
    use crate::msg::{ExecuteMsg, ProtocolSubscriptionData};
    use crate::state::{ExecutionData, SUBSCRIPTIONS, USER_EXECUTION_DATA};

    // Test for query_get_subscriptions
    #[test]
    fn test_query_get_subscriptions() {
        let mut deps = mock_dependencies();

        // Simulate subscriptions
        let user1 = Addr::unchecked("user1");
        let user2 = Addr::unchecked("user2");
        let protocols1 = vec!["AUTO".to_string(), "NAMI".to_string()];
        let protocols2 = vec!["AUTO".to_string()];

        // Insert subscriptions into storage using the Map
        SUBSCRIPTIONS
            .save(deps.as_mut().storage, &user1, &protocols1)
            .unwrap();
        SUBSCRIPTIONS
            .save(deps.as_mut().storage, &user2, &protocols2)
            .unwrap();

        // Call the query
        let result = query_get_subscriptions(deps.as_ref()).unwrap();

        // Find the user1 and user2 entries in the result
        let user1_subs = result
            .subscriptions
            .iter()
            .find(|(addr, _)| addr == "user1")
            .map(|(_, protocols)| protocols);

        let user2_subs = result
            .subscriptions
            .iter()
            .find(|(addr, _)| addr == "user2")
            .map(|(_, protocols)| protocols);

        // Check the response
        assert_eq!(user1_subs.unwrap(), &protocols1);
        assert_eq!(user2_subs.unwrap(), &protocols2);
    }

    #[test]
    fn test_query_get_subscribed_protocols() {
        let mut deps = mock_dependencies();

        // Simulate subscriptions
        let user1 = Addr::unchecked("user1");
        let protocols1 = vec!["AUTO".to_string(), "NAMI".to_string()];

        // Insert user1's subscriptions into storage
        SUBSCRIPTIONS
            .save(deps.as_mut().storage, &user1, &protocols1)
            .unwrap();

        // Simulate execution data for last autoclaim
        let execution_data_auto = ExecutionData {
            last_autoclaim: cosmwasm_std::Timestamp::from_seconds(1633046400),
        };
        let execution_data_nami = ExecutionData {
            last_autoclaim: cosmwasm_std::Timestamp::from_seconds(1633046500),
        };

        // Insert last autoclaim timestamps into USER_EXECUTION_DATA
        USER_EXECUTION_DATA
            .save(
                deps.as_mut().storage,
                (user1.clone(), "AUTO".to_string()),
                &execution_data_auto,
            )
            .unwrap();
        USER_EXECUTION_DATA
            .save(
                deps.as_mut().storage,
                (user1.clone(), "NAMI".to_string()),
                &execution_data_nami,
            )
            .unwrap();

        // Query the specific user's protocols
        let result = query_get_subscribed_protocols(deps.as_ref(), user1).unwrap();

        // Expected data structure
        let expected_protocols = vec![
            ProtocolSubscriptionData {
                protocol: "AUTO".to_string(),
                last_autoclaim: Some(
                    cosmwasm_std::Timestamp::from_seconds(1633046400).seconds(),
                ),
            },
            ProtocolSubscriptionData {
                protocol: "NAMI".to_string(),
                last_autoclaim: Some(
                    cosmwasm_std::Timestamp::from_seconds(1633046500).seconds(),
                ),
            },
        ];

        // Check the response
        assert_eq!(result.protocols, expected_protocols);
    }

    #[test]
    fn test_query_get_subscribed_protocols_empty() {
        let deps = mock_dependencies();

        // Query for a user with no subscriptions
        let result =
            query_get_subscribed_protocols(deps.as_ref(), Addr::unchecked("user1")).unwrap();

        // Check that the result is empty
        assert!(result.protocols.is_empty());
    }

    #[test]
    fn test_query_get_subscriptions_empty() {
        let deps = mock_dependencies();

        // Call the query when there are no subscriptions
        let result = query_get_subscriptions(deps.as_ref()).unwrap();

        // Check that the result is empty
        assert!(result.subscriptions.is_empty());
    }

    #[test]
    fn test_execute_claim_and_stake_with_ignored() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&Addr::unchecked("executor"), &[]);

        // Simulate existing subscriptions
        let user1 = Addr::unchecked("user1");
        let user2 = Addr::unchecked("user2");
        let protocols1 = vec!["AUTO".to_string(), "NAMI".to_string()];

        // Insert subscriptions into storage
        SUBSCRIPTIONS
            .save(deps.as_mut().storage, &user1, &protocols1)
            .unwrap();

        // Prepare users_protocols input, including unsubscribed users and protocols
        let users_protocols = vec![
            (
                user1.to_string(),
                vec!["AUTO".to_string(), "OTHER".to_string()],
            ), // user1 is not subscribed to "OTHER"
            (user2.to_string(), vec!["AUTO".to_string()]), // user2 is not subscribed to any protocols
        ];

        // Execute the claim_and_stake
        let msg = ExecuteMsg::ClaimAndStake { users_protocols };
        let result = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Check the response for ignored users and protocols
        let ignored_attr = result
            .attributes
            .iter()
            .find(|attr| attr.key == "ignored_pairs")
            .expect("Ignored attribute should exist");

        let ignored_list: Vec<String> = serde_json::from_str(&ignored_attr.value).unwrap();

        // Assert that ignored users and protocols are correctly listed
        assert_eq!(
            ignored_list,
            vec![
                format!("{}: OTHER", user1.to_string()), // user1's OTHER protocol should be ignored
                format!("{}: AUTO", user2.to_string()),  // user2's AUTO protocol should be ignored
            ]
        );
    }
}
