use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult, Binary, to_json_binary};

use crate::msg::{
    InstantiateMsg,
    ExecuteMsg,
    QueryMsg,
    ConfigResponse,
};

use crate::state::{
    Config, CONFIG,
};

/// Initializes the contract and stores protocol configurations.
///
/// Stores configurations such as `owner` and protocol settings.
///
/// # Arguments
/// * `_deps` - Mutable dependencies for contract state access.
/// * `_env` - Information about the environment where the contract is running.
/// * `_info` - Information about the sender and funds involved.
/// * `_msg` - The initialization message with config details.
///
/// # Returns
/// A `StdResult<Response>` indicating success or failure.
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        owner: _msg.owner,
    };

    // Save the config in the state
    CONFIG.save(_deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

/// Executes contract logic based on the message received.
///
/// Supports ???.
///
/// # Arguments
/// * `_deps` - Mutable dependencies for contract state access.
/// * `_env` - Information about the environment where the contract is running.
/// * `_info` - Information about the sender and funds involved.
/// * `_msg` - The message specifying the action to execute.
///
/// # Returns
/// A `StdResult<Response>` indicating success or failure.
#[allow(dead_code)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    unimplemented!()
}

/// Handles all query messages in the contract.
///
/// Supported queries include:
/// - `Config`: Retrieves the protocol configuration.
///
/// # Arguments
/// * `_deps` - Dependencies for contract state access.
/// * `_env` - Information about the environment where the contract is running.
/// * `_msg` - The query message specifying the data to retrieve.
///
/// # Returns
/// A `StdResult<Binary>` with the requested data.
pub fn query(
    _deps: Deps, 
    _env: Env, 
    _msg: QueryMsg
) -> StdResult<Binary> {
    match _msg {
        QueryMsg::Config {} => to_json_binary(&query_config(_deps)?),
    }
}

/// Queries the configuration of the protocol stored in the contract.
///
/// # Arguments
/// * `_deps` - Dependencies for contract state access.
///
/// # Returns
/// A `StdResult<ConfigResponse>` containing the protocol configurations.
fn query_config(_deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(_deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner,
    })
}