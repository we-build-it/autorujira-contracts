use crate::error::ContractError;
use crate::msg::{
    ConfigMsg, ConfigResponse, ExecuteMsg, GetSubscribedProtocolsResponse,
    GetSubscriptionsResponse, ProtocolConfig, ProtocolSubscriptionData, QueryMsg,
};
use crate::state::{
    ExecutionData, CONFIG, MAX_PARALLEL_CLAIMS_STORAGE, OWNER, PENDING_USER_PROTOCOL,
    SUBSCRIPTIONS, USER_EXECUTION_DATA,
};
use common::claim::build_claim_msg;
use common::common_functions::query_token_balance;
use common::stake::build_stake_msg;
use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Reply,
    Response, StdResult, SubMsg,
};

const CLAIM_REPLY_ID_BASE: u64 = 100;
const MAX_PARALLEL_CLAIMS: usize = 40;
const FEE_DIVISOR: u128 = 1_000_000_000_000_000_000u128;

/// Checks if the sender is the owner of the contract.
/// If the owner has not been set yet (e.g., during `instantiate`), it returns `Ok()`.
///
/// # Arguments:
/// * `deps` - Dependency injection for accessing state.
/// * `sender` - The address of the sender who initiated the action.
///
/// # Returns:
/// A `Result<(), ContractError>` indicating success if the sender is the owner or if no owner has been set yet.
fn check_is_owner(deps: Deps, sender: &Addr) -> Result<(), ContractError> {
    match OWNER.may_load(deps.storage)? {
        Some(owner) => {
            // If the owner is set, verify that the sender is the owner
            if sender != &owner {
                return Err(ContractError::Unauthorized {});
            }
        }
        None => {
            // If no owner is set yet, assume this is the first call (e.g., during instantiate), so return Ok
            return Ok(());
        }
    }
    Ok(())
}

/// This function initializes the contract and stores protocol configurations.
/// It stores configurations such as `max_parallel_claims` and protocol settings.
///
/// # Arguments:
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `info` - Information about the sender and funds involved.
/// * `msg` - The initialization message with config details.
///
/// # Returns:
/// A `Result<Response, ContractError>` indicating success or failure.
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ConfigMsg,
) -> Result<Response, ContractError> {
    // Ensure that the owner is provided in the instantiate message
    if msg.owner.is_none() {
        return Err(ContractError::NoOwner {});
    }
    update_config(deps, env, info, msg)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

/// Updates the configuration for the specified protocols.
/// It overwrites existing configuration for any protocol provided.
///
/// # Arguments:
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `info` - Information about the sender and funds involved.
/// * `msg` - The update configuration message containing protocol settings.
///
/// # Returns:
/// A `Result<Response, ContractError>` indicating success or failure.
pub fn update_config(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ConfigMsg,
) -> Result<Response, ContractError> {
    // Update the owner if provided (this is where the owner is stored)
    if let Some(owner) = msg.owner {
        OWNER.save(deps.storage, &owner)?;
    }

    let max_parallel_claims = msg.max_parallel_claims.unwrap_or(MAX_PARALLEL_CLAIMS as u8);
    MAX_PARALLEL_CLAIMS_STORAGE.save(deps.storage, &max_parallel_claims)?;

    for protocol_config in msg.protocol_configs {
        CONFIG.save(
            deps.storage,
            protocol_config.protocol.as_str(),
            &protocol_config,
        )?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}

/// Migrates contract data to a new version, if necessary.
///
/// # Arguments:
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `info` - Information about the sender and funds involved.
///
/// # Returns:
/// A `StdResult<Response>` indicating success or failure of the migration.
#[entry_point]
pub fn migrate(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::new()
        .add_attribute("action", "migrate")
        .add_attribute("sender", info.sender)
        .add_attribute("contract_address", env.contract.address))
}

/// Executes contract logic based on the message received. Supports `ClaimAndStake`, `Subscribe`,
/// and `Unsubscribe`.
///
/// # Arguments:
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `info` - Information about the sender and funds involved.
/// * `msg` - The message specifying the action to execute.
///
/// # Returns:
/// A `Result<Response, ContractError>` indicating success or failure.
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            max_parallel_claims,
            protocol_configs,
        } => {
            check_is_owner(deps.as_ref(), &info.sender)?;
            update_config(
                deps,
                env,
                info,
                ConfigMsg {
                    owner,
                    max_parallel_claims,
                    protocol_configs,
                },
            )
        }
        ExecuteMsg::ClaimAndStake { users_protocols } => {
            check_is_owner(deps.as_ref(), &info.sender)?;
            let max_parallel_claims = MAX_PARALLEL_CLAIMS_STORAGE
                .may_load(deps.storage)?
                .unwrap_or(MAX_PARALLEL_CLAIMS as u8);
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
            if total_protocol_count > max_parallel_claims as usize {
                return Err(ContractError::TooManyMessages {
                    max_allowed: max_parallel_claims as usize,
                });
            }
            execute_claim_and_stake(deps, env, users_protocols)
        }
        ExecuteMsg::Subscribe { protocols } => {
            let user = info.sender;
            subscribe(deps, user, protocols)
        }
        ExecuteMsg::Unsubscribe { protocols } => {
            let user = info.sender;
            unsubscribe(deps, user, protocols)
        }
    }
}

/// Claims rewards and stakes them for users across different protocols. Only processes pairs
/// where users are subscribed, ignoring others.
///
/// # Arguments:
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `users_protocols` - A list of (user, protocols) tuples to process.
///
/// # Returns:
/// A `Result<Response, ContractError>` indicating success or failure.
pub fn execute_claim_and_stake(
    deps: DepsMut,
    env: Env,
    users_protocols: Vec<(Addr, Vec<String>)>,
) -> Result<Response, ContractError> {
    let mut messages: Vec<SubMsg> = vec![];
    let mut ignored_pairs: Vec<(Addr, String)> = vec![];

    let mut reply_id = CLAIM_REPLY_ID_BASE;

    for (user, protocols) in users_protocols {
        let user_subscriptions = SUBSCRIPTIONS
            .may_load(deps.storage, &user)?
            .unwrap_or_default();

        for protocol in protocols {
            if !user_subscriptions.contains(&protocol) {
                ignored_pairs.push((user.clone(), protocol.clone()));
                continue;
            }

            let protocol_config = CONFIG.may_load(deps.storage, &protocol)?.ok_or(
                ContractError::InvalidProtocol {
                    protocol: protocol.to_string(),
                },
            )?;

            let balance_before =
                query_token_balance(deps.as_ref(), &user, protocol_config.reward_denom.clone())?;

            PENDING_USER_PROTOCOL.save(
                deps.storage,
                reply_id,
                &(user.clone(), protocol.clone(), balance_before),
            )?;

            let claim_contract_addr = deps
                .api
                .addr_validate(&protocol_config.claim_contract_address)?;

            let claim_msg = build_claim_msg(
                env.clone(),
                user.clone(),
                protocol_config.provider.clone(),
                claim_contract_addr,
                2, // Example claim ID
            )?;

            let submsg = SubMsg::reply_on_success(claim_msg, reply_id);
            messages.push(submsg);
            reply_id += 1;

            let execution_data = ExecutionData {
                last_autoclaim: env.block.time,
            };

            USER_EXECUTION_DATA.save(
                deps.storage,
                (user.clone(), protocol.clone()),
                &execution_data,
            )?;
        }
    }

    Ok(Response::new()
        .add_submessages(messages)
        .add_attribute("action", "execute_claim_and_stake")
        .add_attribute("ignored_count", ignored_pairs.len().to_string())
        .add_attribute("ignored_pairs", format!("{:?}", ignored_pairs)))
}

/// Handles the response after a claim has been processed. Stakes the rewards based on the
/// result of the claim.
///
/// # Arguments:
/// * `deps` - Mutable dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `msg` - The reply message after claim execution.
///
/// # Returns:
/// A `Result<Response, ContractError>` indicating success or failure.
#[entry_point]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    if let Some((user, protocol, balance_before)) = PENDING_USER_PROTOCOL
        .may_load(deps.storage, msg.id)
        .unwrap_or_default()
    {
        let protocol_config =
            CONFIG
                .may_load(deps.storage, &protocol)?
                .ok_or(ContractError::InvalidProtocol {
                    protocol: protocol.to_string(),
                })?;

        let balance_after =
            query_token_balance(deps.as_ref(), &user, protocol_config.reward_denom.clone())?;

        let amount_claimed =
            balance_after
                .checked_sub(balance_before)
                .map_err(|_| ContractError::NoRewards {
                    msg: "No rewards claimed".to_string(),
                })?;

        let fee_amount =
            amount_claimed.multiply_ratio(protocol_config.fee_percentage.atomics(), FEE_DIVISOR);
        let stake_amount =
            amount_claimed
                .checked_sub(fee_amount)
                .map_err(|_| ContractError::NoRewards {
                    msg: "Stake amount is zero".to_string(),
                })?;

        let stake_contract_addr = deps
            .api
            .addr_validate(&protocol_config.stake_contract_address)?;

        let stake_msg = build_stake_msg(
            env.clone(),
            user.clone(),
            protocol_config.provider.clone(),
            stake_contract_addr,
            stake_amount.u128(),
            protocol_config.reward_denom.clone(),
        )?;

        // TODO build fee send msg if fee_amount > 0

        let event = Event::new("claim_and_stake")
            .add_attribute("address", user.to_string())
            .add_attribute("claimed", amount_claimed.to_string())
            .add_attribute("fee", fee_amount.to_string())
            .add_attribute("staked", stake_amount.to_string())
            .add_attribute("timestamp", env.block.time.seconds().to_string());

        Ok(Response::new()
            .add_message(stake_msg)
            .add_event(event)
            .add_attribute("action", "reply_claim_and_stake"))
    } else {
        Err(ContractError::InvalidReplyId { id: msg.id })
    }
}

/// Subscribes a user to the specified protocols.
///
/// # Arguments:
/// * `deps` - Mutable dependencies for contract state access.
/// * `user` - The address of the user subscribing.
/// * `protocols` - A list of protocol names the user subscribes to.
///
/// # Returns:
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
/// # Arguments:
/// * `deps` - Mutable dependencies for contract state access.
/// * `user` - The address of the user unsubscribing.
/// * `protocols` - A list of protocol names to unsubscribe from.
///
/// # Returns:
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
/// # Arguments:
/// * `deps` - Dependencies for contract state access.
///
/// # Returns:
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
/// # Arguments:
/// * `deps` - Dependencies for contract state access.
/// * `user` - The address of the user.
///
/// # Returns:
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

/// Handles all query messages in the contract. Supported queries include:
/// - `Config`: Retrieves the protocol configuration.
/// - `GetSubscriptions`: Retrieves all user subscriptions.
/// - `GetSubscribedProtocols`: Retrieves a specific user's subscriptions.
///
/// # Arguments:
/// * `deps` - Dependencies for contract state access.
/// * `env` - Information about the environment where the contract is running.
/// * `msg` - The query message specifying the data to retrieve.
///
/// # Returns:
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
/// # Arguments:
/// * `deps` - Dependencies for contract state access.
///
/// # Returns:
/// A `StdResult<ConfigResponse>` containing the protocol configurations.
fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let max_parallel_claims = MAX_PARALLEL_CLAIMS_STORAGE
        .may_load(deps.storage)?
        .unwrap_or(MAX_PARALLEL_CLAIMS as u8);

    let configs: Vec<ProtocolConfig> = CONFIG
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| item.map(|(_, config)| config))
        .collect::<StdResult<Vec<ProtocolConfig>>>()?;

    Ok(ConfigResponse {
        owner: OWNER.may_load(deps.storage)?,
        max_parallel_claims: Some(max_parallel_claims),
        protocol_configs: configs,
    })
}