use cosmwasm_std::{to_json_binary, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg};

use crate::msg::{
    InstantiateMsg,
    ExecuteMsg,
    QueryMsg,
    ConfigResponse,
};

use crate::state::{
    Config, PoolKey, UserOrder, CONFIG, FIN_CONTRACTS, USER_ORDERS
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

pub const FIN_REPLY_SUBMIT_ORDER: u64 = 200;
pub const FIN_REPLY_SUBMIT_CLAIM_ORDER: u64 = 201;
pub const FIN_REPLY_SUBMIT_SWAP_SLTP: u64 = 202;

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
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::PlaceOrder { 
            fin_contract_address,
            side, // TODO: Should we ask for the side or should we infer it from the funds?
            price, 
            amount, // TODO: Should we ask for the amount or should we infer it from the funds?
            price_sl, 
            price_tp,
        } => {
            // Ensure the contract is valid
            let denoms = FIN_CONTRACTS.load(deps.storage, &fin_contract_address)?;

            // Check if the user has a previous order at that side and price
            let user_order_key = (
                info.sender.clone(), 
                PoolKey::new(side.clone(), price.clone())
            );
            let old_order = USER_ORDERS.may_load(
                deps.storage, 
                user_order_key.clone());

            // TODO: Ensure funds are OK taking into account old_older

            // Send Submit Order message to FIN
            let mut submessages: Vec<SubMsg> = Vec::new();            
            let mut reply_id = FIN_REPLY_SUBMIT_ORDER;
            let msg = rujira_rs::fin::ExecuteMsg::Order(vec![(side, price, amount)]);
            let execute_msg = WasmMsg::Execute {
                contract_addr: fin_contract_address.to_string(),
                msg: to_json_binary(&msg)?,
                funds: info.funds,
            };

            let mut response = Response::new().add_event(
                cosmwasm_std::Event::new("autorujira.autosltp")
                    .add_attribute("action", "place_order")
                    .add_attribute("sender", info.sender.to_string()),
            );
            
            // TODO: Is this really necessary?
            response = response.add_submessage(SubMsg::reply_on_success(execute_msg, reply_id.clone()));

            USER_ORDERS.save(
                deps.storage, 
                user_order_key.clone(), 
                &UserOrder {amount, price_sl, price_tp}
            )?;
        
            // create attributes, events, and fund transfeer for sender
            return Ok(response);          
        }
        ExecuteMsg::Protect { 
            fin_contract_address, 
            side, 
            price 
        } => {
            // Ensure the contract is valid
            let denoms = FIN_CONTRACTS.load(deps.storage, &fin_contract_address)?;

            // Check if the user has a previous order at that side and price
            let user_order_key = (
                info.sender.clone(), 
                PoolKey::new(side.clone(), price.clone())
            );
            let user_order = USER_ORDERS.load(
                deps.storage, 
                user_order_key.clone())?;

            let current_price = load_oracle_price(denoms.base(), denoms.quote())?;

            if (user_order.price_sl.is_some() && user_order.price_sl.unwrap() >= current_price) ||
               (user_order.price_tp.is_some() && user_order.price_tp.unwrap() <= current_price) {
                // First Claim the order
                let mut reply_id = FIN_REPLY_SUBMIT_CLAIM_ORDER;

                let msg_claim = rujira_rs::fin::ExecuteMsg::Order(vec![(side, price, Uint128::zero())]);
                let execute_msg_claim = WasmMsg::Execute {
                    contract_addr: fin_contract_address.to_string(),
                    msg: to_json_binary(&msg_claim)?,
                    funds: vec![Coin::new(Uint128::zero(), denoms.quote())],
                };
                // TODO: take claimed funds from previous execution
                let claimed_funds = vec![Coin::new(Uint128::zero(), denoms.quote())];

                let msg_swap = rujira_rs::fin::ExecuteMsg::Swap {
                    min_return: None,
                    to: None,
                    callback: None,
                };
                let execute_msg_swap = WasmMsg::Execute {
                    contract_addr: fin_contract_address.to_string(),
                    msg: to_json_binary(&msg_swap)?,
                    funds: claimed_funds,
                };
    
                let mut response = Response::new().add_event(
                    cosmwasm_std::Event::new("autorujira.autosltp")
                        .add_attribute("action", "claim_order")
                        .add_attribute("sender", info.sender.to_string()),
                );
                return Ok(response)
            }
            return Err(StdError::generic_err("SL/TP not reached yet"));
        },      

    }
}

fn load_oracle_price(_base: &str, _quote: &str) -> StdResult<Decimal> {
    // TODO: Implement oracle price get
    // let a = query::Pool::load(q, &config.oracles[0])?.asset_tor_price;
    // let b = query::Pool::load(q, &config.oracles[1])?.asset_tor_price;
    // Ok(a / b)
    Ok(Decimal::zero())
}

pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
    match reply.id {
        id if id == FIN_REPLY_SUBMIT_CLAIM_ORDER => {
            Ok(Response::new())
        }
        // --- error
        _ => {
            return Err(StdError::generic_err("unknown reply id".to_string()))
        }
    }
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