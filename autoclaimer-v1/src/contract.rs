use crate::error::ContractError;
#[cfg(test)]
use crate::mocks::mock_functions::{
    build_FIN_claim_msg, build_claim_msg, build_send_msg, build_stake_msg,
};
#[cfg(not(test))]
use common::claim::{build_FIN_claim_msg, build_claim_msg};
#[cfg(not(test))]
use common::send::build_send_msg;
#[cfg(not(test))]
use common::stake::build_stake_msg;
use cw_storage_plus::Map;

use crate::msg::{
    ConfigResponse, ExecuteMsg, GetSubscribedProtocolsResponse, GetSubscriptionsResponse,
    InstantiateMsg, OldProtocolConfig, ProtocolConfig, ProtocolStrategy, ProtocolSubscriptionData,
    QueryMsg, UpdateConfigMsg,
};
use crate::state::{
    Config, ExecutionData, CONFIG, PENDING_CLAIM_AND_STAKE_DATA, PENDING_CLAIM_ONLY_DATA,
    PROTOCOL_CONFIG, SUBSCRIPTIONS, USER_EXECUTION_DATA,
};

use common::common_functions::query_token_balance;
use cosmwasm_std::{
    ensure, entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo,
    Reply, ReplyOn, Response, StdResult, SubMsg,
};
use cw_utils::nonpayable;

/// Enum representing the result of an action.
#[derive(Debug, Clone, Copy)]
enum ActionResult {
    Ok,
    Failed,
}

impl ActionResult {
    fn as_str(&self) -> &'static str {
        match self {
            ActionResult::Ok => "ok",
            ActionResult::Failed => "failed",
        }
    }
}

// Constants for reply IDs
const CLAIM_AND_STAKE_CLAIM_BASE_ID: u64 = 1000;
const CLAIM_AND_STAKE_STAKE_BASE_ID: u64 = 2000;
const CLAIM_AND_STAKE_SEND_BASE_ID: u64 = 3000;
const CLAIM_ONLY_CLAIM_BASE_ID: u64 = 4000;
const FEE_DIVISOR: u128 = 1_000_000_000_000_000_000u128;

/// Helper function to validate protocols.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `protocols` - A list of protocol names to validate.
///
/// # Returns
/// A `Result<(), ContractError>` indicating success or failure.
fn validate_protocols(deps: &DepsMut, protocols: &Vec<String>) -> Result<(), ContractError> {
    for protocol in protocols {
        if PROTOCOL_CONFIG.may_load(deps.storage, protocol)?.is_none() {
            return Err(ContractError::InvalidProtocol {
                protocol: protocol.clone(),
            });
        }
    }
    Ok(())
}

/// Initializes the contract and stores protocol configurations.
///
/// Stores configurations such as `max_parallel_claims` and protocol settings.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `_env` - Information about the environment where the contract is running.
/// * `_info` - Information about the sender and funds involved.
/// * `msg` - The initialization message with config details.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: msg.owner,
        max_parallel_claims: msg.max_parallel_claims,
    };

    // Save the config in the state
    CONFIG.save(deps.storage, &config)?;

    for protocol_config in msg.protocol_configs {
        PROTOCOL_CONFIG.save(
            deps.storage,
            protocol_config.protocol.as_str(),
            &protocol_config,
        )?;
    }

    Ok(Response::new().add_attribute("action", "instantiate"))
}

// Define the old Map with the same storage prefix
const OLD_PROTOCOL_CONFIG: Map<&str, OldProtocolConfig> = Map::new("protocol_config");

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {
    // Load the existing global configuration
    let old_config = CONFIG.load(deps.storage)?;

    // Get all the keys from the old protocol config
    let keys: Vec<String> = OLD_PROTOCOL_CONFIG
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    // Iterate over each key to migrate data
    for protocol in keys {
        // Load old data using the old map
        let old_data = OLD_PROTOCOL_CONFIG.load(deps.storage, &protocol)?;

        // Construct the new strategy based on the old data
        let new_strategy = ProtocolStrategy::ClaimAndStakeDaoDaoCwRewards {
            provider: old_data.provider,
            claim_contract_address: old_data.claim_contract_address,
            stake_contract_address: old_data.stake_contract_address,
            reward_denom: old_data.reward_denom,
        };

        // Create the new protocol configuration
        let new_protocol_config = ProtocolConfig {
            protocol: protocol.clone(),
            fee_percentage: old_data.fee_percentage,
            fee_address: old_data.fee_address,
            strategy: new_strategy,
        };

        // Save the new configuration using the new map
        PROTOCOL_CONFIG.save(deps.storage, &protocol, &new_protocol_config)?;
    }

    // Save the updated global configuration
    CONFIG.save(deps.storage, &old_config)?;

    Ok(Response::new().add_attribute("action", "migrate_protocols"))
}

/// Updates the configuration for the specified protocols.
///
/// It overwrites existing configuration for any protocol provided.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `_env` - Information about the environment where the contract is running.
/// * `info` - Information about the sender and funds involved.
/// * `msg` - The update configuration message containing protocol settings.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    ensure!(config.owner == info.sender, ContractError::Unauthorized {});

    // Update the owner if provided
    if let Some(owner) = msg.owner {
        config.owner = owner;
    }

    // Update the max parallel claims if provided
    if let Some(max_parallel_claims) = msg.max_parallel_claims {
        config.max_parallel_claims = max_parallel_claims;
    }

    CONFIG.save(deps.storage, &config)?;

    if let Some(protocol_configs) = msg.protocol_configs {
        for protocol_config in protocol_configs {
            PROTOCOL_CONFIG.save(
                deps.storage,
                protocol_config.protocol.as_str(),
                &protocol_config,
            )?;
        }
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}

/// Executes contract logic based on the message received.
///
/// Supports `ClaimAndStake`, `Subscribe`, and `Unsubscribe`.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `info` - Information about the sender and funds involved.
/// * `msg` - The message specifying the action to execute.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info).map_err(|_| ContractError::GenericError {
        msg: "Don't send funds to this function!".to_string(),
    })?;

    match msg {
        ExecuteMsg::UpdateConfig {
            config: update_config_msg,
        } => update_config(deps, env, info, update_config_msg),
        ExecuteMsg::ClaimAndStake { users_protocols } => {
            let config = CONFIG.load(deps.storage)?;
            ensure!(config.owner == info.sender, ContractError::Unauthorized {});

            let mut total_protocol_count = 0;
            let users_protocols: Vec<(Addr, Vec<String>)> = users_protocols
                .into_iter()
                .map(|(user_string, protocols)| {
                    let user_addr = deps.api.addr_validate(&user_string)?;
                    total_protocol_count += protocols.len();
                    Ok((user_addr, protocols))
                })
                .collect::<Result<Vec<(Addr, Vec<String>)>, ContractError>>()?;

            // Validation: Check the total number of protocols to process
            if total_protocol_count > config.max_parallel_claims as usize {
                return Err(ContractError::TooManyMessages {
                    max_allowed: config.max_parallel_claims as usize,
                });
            }

            execute_claim_and_stake(deps, env, users_protocols)
        }
        ExecuteMsg::ClaimOnly {
            protocol,
            users_contracts,
        } => {
            let config = CONFIG.load(deps.storage)?;
            ensure!(config.owner == info.sender, ContractError::Unauthorized {});
            if users_contracts.len() > config.max_parallel_claims as usize {
                return Err(ContractError::TooManyMessages {
                    max_allowed: config.max_parallel_claims as usize,
                });
            }
            execute_claim_only(deps, env, info, protocol, users_contracts)
        }
        ExecuteMsg::Subscribe { protocols } => {
            validate_protocols(&deps, &protocols)?;
            let user = info.sender;
            subscribe(deps, user, protocols)
        }
        ExecuteMsg::Unsubscribe { protocols } => {
            validate_protocols(&deps, &protocols)?;
            let user = info.sender;
            unsubscribe(deps, user, protocols)
        }
    }
}

/// Claims rewards and stakes them for users across different protocols.
///
/// Only processes pairs where users are subscribed, ignoring others.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `users_protocols` - A list of (user, protocols) tuples to process.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
pub fn execute_claim_and_stake(
    deps: DepsMut,
    env: Env,
    users_protocols: Vec<(Addr, Vec<String>)>,
) -> Result<Response, ContractError> {
    let mut messages: Vec<SubMsg> = vec![];
    let mut ignored_pairs: Vec<(Addr, String)> = vec![];

    for (user, protocols) in users_protocols {
        let user_subscriptions = SUBSCRIPTIONS
            .may_load(deps.storage, &user)?
            .unwrap_or_default();

        for protocol in protocols {
            if !user_subscriptions.contains(&protocol) {
                ignored_pairs.push((user.clone(), protocol.clone()));
                continue;
            }

            let protocol_config = PROTOCOL_CONFIG.may_load(deps.storage, &protocol)?.ok_or(
                ContractError::InvalidProtocol {
                    protocol: protocol.clone(),
                },
            )?;

            match protocol_config.strategy {
                ProtocolStrategy::ClaimAndStakeDaoDaoCwRewards {
                    ref provider,
                    ref claim_contract_address,
                    stake_contract_address: _,
                    ref reward_denom,
                } => {
                    let balance_before =
                        query_token_balance(deps.as_ref(), &user, reward_denom.to_string())?;

                    // Save pending protocol data for processing in the reply
                    PENDING_CLAIM_AND_STAKE_DATA.save(
                        deps.storage,
                        CLAIM_AND_STAKE_CLAIM_BASE_ID + messages.len() as u64,
                        &(user.clone(), protocol.clone(), balance_before),
                    )?;

                    let claim_contract_addr = deps.api.addr_validate(claim_contract_address)?;

                    // Create claim message
                    let claim_msg = build_claim_msg(
                        env.clone(),
                        user.clone(),
                        provider.clone(),
                        claim_contract_addr,
                        2, // Example claim ID
                    )?;

                    let submsg = SubMsg {
                        msg: claim_msg,
                        gas_limit: None,
                        id: CLAIM_AND_STAKE_CLAIM_BASE_ID + messages.len() as u64,
                        reply_on: ReplyOn::Always,
                    };

                    messages.push(submsg);
                }
                _ => {
                    ignored_pairs.push((user.clone(), protocol.clone()));
                }
            }
        }
    }

    let event = Event::new("autorujira.autoclaimer")
        .add_attribute("action", "execute_claim_and_stake")
        .add_attribute("ignored_count", ignored_pairs.len().to_string())
        .add_attribute("ignored_pairs", format!("{:?}", ignored_pairs));

    Ok(Response::new().add_submessages(messages).add_event(event))
}

/// Handles the response after any submessage has been processed.
///
/// The type of action (claim, stake, send) is determined by the reply ID.
/// Events for `ok` or `failed` results are emitted accordingly.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `msg` - The reply message after execution.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id >= CLAIM_AND_STAKE_CLAIM_BASE_ID && msg.id < CLAIM_AND_STAKE_STAKE_BASE_ID {
        process_claim_and_stake_claim_reply(deps, env, msg)
    } else if msg.id >= CLAIM_AND_STAKE_STAKE_BASE_ID && msg.id < CLAIM_AND_STAKE_SEND_BASE_ID {
        process_claim_and_stake_stake_reply(msg)
    } else if msg.id >= CLAIM_AND_STAKE_SEND_BASE_ID && msg.id < CLAIM_ONLY_CLAIM_BASE_ID {
        process_claim_and_stake_send_reply(msg)
    } else if msg.id >= CLAIM_ONLY_CLAIM_BASE_ID {
        process_claim_only_claim_reply(deps, env, msg)
    } else {
        Err(ContractError::InvalidReplyId { id: msg.id })
    }
}

/// Processes the reply for a claim message.
///
/// Emits an event indicating whether the claim was successful or failed.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `msg` - The reply message after claim execution.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
fn process_claim_and_stake_claim_reply(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    if let Some((user, protocol, balance_before)) =
        PENDING_CLAIM_AND_STAKE_DATA.may_load(deps.storage, msg.id)?
    {
        let protocol_config = PROTOCOL_CONFIG.load(deps.storage, &protocol)?;

        let msg_id_str = msg.id.to_string();
        let mut attributes = vec![
            ("protocol", protocol.clone()),
            ("address", user.to_string()),
        ];

        let mut submessages = vec![];
        let mut claim_result = ActionResult::Ok;

        match msg.result {
            cosmwasm_std::SubMsgResult::Ok(_) => {
                let reward_denom = match &protocol_config.strategy {
                    ProtocolStrategy::ClaimAndStakeDaoDaoCwRewards { reward_denom, .. } => {
                        reward_denom
                    }
                    _ => {
                        return Err(ContractError::InvalidStrategy {
                            strategy: protocol_config.strategy.as_str().to_string(),
                        })
                    }
                };

                let balance_after =
                    query_token_balance(deps.as_ref(), &user, reward_denom.clone())?;

                let amount_claimed = balance_after.checked_sub(balance_before).map_err(|_| {
                    ContractError::NoRewards {
                        msg: "No rewards claimed".to_string(),
                    }
                })?;

                let fee_amount = amount_claimed
                    .multiply_ratio(protocol_config.fee_percentage.atomics(), FEE_DIVISOR);

                let stake_amount = amount_claimed.checked_sub(fee_amount).map_err(|_| {
                    ContractError::NoRewards {
                        msg: "Stake amount is zero".to_string(),
                    }
                })?;

                // Handle ClaimAndStakeDaoDaoCwRewards strategy
                if let ProtocolStrategy::ClaimAndStakeDaoDaoCwRewards {
                    provider,
                    stake_contract_address,
                    ..
                } = &protocol_config.strategy
                {
                    // Create stake message
                    let stake_msg = build_stake_msg(
                        env.clone(),
                        user.clone(),
                        provider.clone(),
                        deps.api.addr_validate(stake_contract_address)?,
                        stake_amount.u128(),
                        reward_denom.clone(),
                    )?;

                    // Create send fee message if fee > 0
                    if fee_amount > 0u128.into() {
                        let send_msg = build_send_msg(
                            env.clone(),
                            user.clone(),
                            deps.api.addr_validate(&protocol_config.fee_address)?,
                            fee_amount.u128(),
                            reward_denom.clone(),
                        )?;

                        submessages.push(SubMsg {
                            msg: send_msg,
                            gas_limit: None,
                            id: CLAIM_AND_STAKE_SEND_BASE_ID + msg.id
                                - CLAIM_AND_STAKE_CLAIM_BASE_ID,
                            reply_on: ReplyOn::Always,
                        });
                    }

                    // Add submessages
                    submessages.push(SubMsg {
                        msg: stake_msg,
                        gas_limit: None,
                        id: CLAIM_AND_STAKE_STAKE_BASE_ID + msg.id - CLAIM_AND_STAKE_CLAIM_BASE_ID,
                        reply_on: ReplyOn::Always,
                    });

                    // Add attributes for success
                    attributes.push(("token", reward_denom.to_string()));
                    attributes.push(("tokens_claimed", amount_claimed.to_string()));
                    attributes.push(("fee_to_charge", fee_amount.to_string()));
                    attributes.push(("tokens_to_stake", stake_amount.to_string()));
                    attributes.push(("timestamp", env.block.time.seconds().to_string()));

                    // Save last autoclaim
                    let execution_data = ExecutionData {
                        last_autoclaim: env.block.time,
                    };

                    USER_EXECUTION_DATA.save(
                        deps.storage,
                        (user.clone(), protocol_config.protocol.clone()),
                        &execution_data,
                    )?;
                }
            }
            cosmwasm_std::SubMsgResult::Err(err) => {
                attributes.push(("error", err.clone()));
                claim_result = ActionResult::Failed;
            }
        }

        // Create a single event with attributes
        let event = Event::new("autorujira.autoclaimer")
            .add_attribute("action", "claim")
            .add_attribute("msg_id", msg_id_str)
            .add_attribute("result", claim_result.as_str())
            .add_attributes(attributes);

        // Return the final response with submessages and event
        Ok(Response::new()
            .add_submessages(submessages)
            .add_event(event))
    } else {
        Err(ContractError::InvalidReplyId { id: msg.id })
    }
}

/// Processes the reply for a stake message.
///
/// Emits an event indicating whether the stake was successful or failed.
///
/// # Arguments
/// * `msg` - The reply message after stake execution.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
fn process_claim_and_stake_stake_reply(msg: Reply) -> Result<Response, ContractError> {
    let mut event = Event::new("autorujira.autoclaimer")
        .add_attribute("action", "stake")
        .add_attribute("msg_id", msg.id.to_string());

    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            event = event.add_attribute("result", ActionResult::Ok.as_str());
        }
        cosmwasm_std::SubMsgResult::Err(err) => {
            event = event.add_attribute("result", ActionResult::Failed.as_str());
            event = event.add_attribute("error", err.as_str());
        }
    }

    Ok(Response::new().add_event(event))
}

/// Processes the reply for a send fee message.
///
/// Emits an event indicating whether the send was successful or failed.
///
/// # Arguments
/// * `msg` - The reply message after send execution.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
fn process_claim_and_stake_send_reply(msg: Reply) -> Result<Response, ContractError> {
    let mut event = Event::new("autorujira.autoclaimer")
        .add_attribute("action", "charge_fee")
        .add_attribute("msg_id", msg.id.to_string());

    match msg.result {
        cosmwasm_std::SubMsgResult::Ok(_) => {
            event = event.add_attribute("result", ActionResult::Ok.as_str());
        }
        cosmwasm_std::SubMsgResult::Err(err) => {
            event = event.add_attribute("result", ActionResult::Failed.as_str());
            event = event.add_attribute("error", err.as_str());
        }
    }

    Ok(Response::new().add_event(event))
}

/// Executes claim-only actions for specified users and contracts.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `info` - Information about the sender and funds involved.
/// * `protocol` - The protocol name.
/// * `users_contracts` - A list of (user, contract_address) tuples.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
pub fn execute_claim_only(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    protocol: String,
    users_contracts: Vec<(String, String)>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure!(config.owner == info.sender, ContractError::Unauthorized {});

    let protocol_config = PROTOCOL_CONFIG.load(deps.storage, &protocol)?;

    // Verify that the strategy supports claim_only
    match protocol_config.strategy {
        ProtocolStrategy::ClaimOnlyFIN {
            ref supported_markets,
        } => {
            let mut messages: Vec<SubMsg> = vec![];
            let mut ignored_markets: Vec<(String, String)> = vec![];

            for (user_string, contract_address) in users_contracts {
                if !supported_markets.contains(&contract_address) {
                    ignored_markets.push((user_string.clone(), contract_address.clone()));
                    continue;
                }

                let user = deps.api.addr_validate(&user_string)?;
                let contract_addr = deps.api.addr_validate(&contract_address)?;

                // Build the claim message
                let claim_msg =
                    build_FIN_claim_msg(env.clone(), user.clone(), contract_addr.clone())?;

                // Create SubMsg with unique ID
                let msg_id = CLAIM_ONLY_CLAIM_BASE_ID + messages.len() as u64;

                PENDING_CLAIM_ONLY_DATA.save(
                    deps.storage,
                    msg_id,
                    &(protocol.clone(), user.clone(), contract_addr.clone()),
                )?;

                let submsg = SubMsg {
                    msg: claim_msg,
                    gas_limit: None,
                    id: msg_id,
                    reply_on: ReplyOn::Always,
                };

                messages.push(submsg);
            }

            let event = Event::new("autorujira.autoclaimer")
                .add_attribute("action", "execute_claim_only")
                .add_attribute("ignored_count", ignored_markets.len().to_string())
                .add_attribute("ignored_markets", format!("{:?}", ignored_markets));

            Ok(Response::new().add_submessages(messages).add_event(event))
        }
        _ => Err(ContractError::InvalidStrategy {
            strategy: protocol_config.strategy.as_str().to_string(),
        }),
    }
}

/// Processes the reply for a claim-only message.
///
/// Emits an event indicating whether the claim was successful or failed.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `msg` - The reply message after claim execution.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
fn process_claim_only_claim_reply(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    if let Some((protocol, user, contract_address)) =
        PENDING_CLAIM_ONLY_DATA.may_load(deps.storage, msg.id)?
    {
        let msg_id_str = msg.id.to_string();
        let mut attributes = vec![
            ("protocol".to_string(), protocol.clone()),
            ("address".to_string(), user.to_string()),
            ("contract_address".to_string(), contract_address.to_string()),
        ];

        let mut claim_result = ActionResult::Ok;

        match msg.result {
            cosmwasm_std::SubMsgResult::Ok(_) => {
                // Add the timestamp as an additional attribute
                attributes.push((
                    "timestamp".to_string(),
                    env.block.time.seconds().to_string(),
                ));

                // Save last autoclaim
                let execution_data = ExecutionData {
                    last_autoclaim: env.block.time,
                };

                USER_EXECUTION_DATA.save(
                    deps.storage,
                    (user.clone(), protocol.clone()),
                    &execution_data,
                )?;
            }
            cosmwasm_std::SubMsgResult::Err(err) => {
                attributes.push(("error".to_string(), err.clone()));
                claim_result = ActionResult::Failed;
            }
        }

        // Create the main event
        let event = Event::new("autorujira.autoclaimer")
            .add_attribute("action", "claim")
            .add_attribute("msg_id", msg_id_str)
            .add_attribute("result", claim_result.as_str())
            .add_attributes(attributes);

        Ok(Response::new().add_event(event))
    } else {
        Err(ContractError::InvalidReplyId { id: msg.id })
    }
}

/// Subscribes a user to the specified protocols.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `user` - The address of the user subscribing.
/// * `protocols` - A list of protocol names the user subscribes to.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
pub fn subscribe(
    deps: DepsMut,
    user: Addr,
    protocols: Vec<String>,
) -> Result<Response, ContractError> {
    let mut user_subscriptions = SUBSCRIPTIONS
        .may_load(deps.storage, &user)?
        .unwrap_or_default();

    for protocol in protocols {
        if !user_subscriptions.contains(&protocol) {
            user_subscriptions.push(protocol);
        }
    }

    SUBSCRIPTIONS.save(deps.storage, &user, &user_subscriptions)?;

    Ok(Response::new()
        .add_attribute("action", "subscribe")
        .add_attribute("user", user.to_string())
        .add_attribute("subscribed_protocols", format!("{:?}", user_subscriptions)))
}

/// Unsubscribes a user from the specified protocols.
///
/// # Arguments
/// * `deps` - Mutable dependencies for contract state access.
/// * `user` - The address of the user unsubscribing.
/// * `protocols` - A list of protocol names to unsubscribe from.
///
/// # Returns
/// A `Result<Response, ContractError>` indicating success or failure.
pub fn unsubscribe(
    deps: DepsMut,
    user: Addr,
    protocols: Vec<String>,
) -> Result<Response, ContractError> {
    let mut user_subscriptions = SUBSCRIPTIONS.load(deps.storage, &user)?;

    for protocol in protocols {
        if let Some(index) = user_subscriptions.iter().position(|p| p == &protocol) {
            user_subscriptions.remove(index);
        }
    }

    SUBSCRIPTIONS.save(deps.storage, &user, &user_subscriptions)?;

    Ok(Response::new()
        .add_attribute("action", "unsubscribe")
        .add_attribute("user", user.to_string()))
}

/// Queries all user subscriptions stored in the contract.
///
/// # Arguments
/// * `deps` - Dependencies for contract state access.
///
/// # Returns
/// A `StdResult<GetSubscriptionsResponse>` containing the list of subscriptions.
pub fn query_get_subscriptions(deps: Deps) -> StdResult<GetSubscriptionsResponse> {
    let subscriptions: Vec<_> = SUBSCRIPTIONS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            let (addr, protocols) = item?;
            Ok((addr.to_string(), protocols))
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(GetSubscriptionsResponse { subscriptions })
}

/// Queries the protocols that a specific user is subscribed to.
///
/// # Arguments
/// * `deps` - Dependencies for contract state access.
/// * `user` - The address of the user.
///
/// # Returns
/// A `StdResult<GetSubscribedProtocolsResponse>` containing the user's subscriptions.
pub fn query_get_subscribed_protocols(
    deps: Deps,
    user: Addr,
) -> StdResult<GetSubscribedProtocolsResponse> {
    let user_subscriptions = SUBSCRIPTIONS
        .may_load(deps.storage, &user)?
        .unwrap_or_default();

    let mut protocols_data = Vec::new();

    for protocol in user_subscriptions {
        let execution_data =
            USER_EXECUTION_DATA.may_load(deps.storage, (user.clone(), protocol.clone()))?;

        let last_autoclaim = execution_data.map(|data| data.last_autoclaim.seconds());

        protocols_data.push(ProtocolSubscriptionData {
            protocol,
            last_autoclaim,
        });
    }

    Ok(GetSubscribedProtocolsResponse {
        protocols: protocols_data,
    })
}

/// Handles all query messages in the contract.
///
/// Supported queries include:
/// - `Config`: Retrieves the protocol configuration.
/// - `GetSubscriptions`: Retrieves all user subscriptions.
/// - `GetSubscribedProtocols`: Retrieves a specific user's subscriptions.
///
/// # Arguments
/// * `deps` - Dependencies for contract state access.
/// * `_env` - Information about the environment where the contract is running.
/// * `msg` - The query message specifying the data to retrieve.
///
/// # Returns
/// A `StdResult<Binary>` with the requested data.
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::GetSubscriptions {} => to_json_binary(&query_get_subscriptions(deps)?),
        QueryMsg::GetSubscribedProtocols { user_address } => {
            let user_addr = deps.api.addr_validate(&user_address)?;
            to_json_binary(&query_get_subscribed_protocols(deps, user_addr)?)
        }
    }
}

/// Queries the configuration of the protocol stored in the contract.
///
/// # Arguments
/// * `deps` - Dependencies for contract state access.
///
/// # Returns
/// A `StdResult<ConfigResponse>` containing the protocol configurations.
fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let protocol_configs: Vec<ProtocolConfig> = PROTOCOL_CONFIG
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| item.map(|(_, config)| config))
        .collect::<StdResult<Vec<ProtocolConfig>>>()?;

    Ok(ConfigResponse {
        owner: config.owner,
        max_parallel_claims: config.max_parallel_claims,
        protocol_configs,
    })
}
