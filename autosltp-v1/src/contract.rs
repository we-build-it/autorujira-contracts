use cosmwasm_std::{to_json_binary, Addr, BankMsg, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, SubMsg, SubMsgResponse, SubMsgResult, Uint128, WasmMsg};

use crate::msg::{
    InstantiateMsg,
    ExecuteMsg,
    QueryMsg,
    ConfigResponse,
};

use crate::state::{
    Config, PoolKey, UserOrder, CONFIG, FIN_CONTRACTS, IN_FLIGHT_USER, USER_ORDERS
};

use rujira_rs::fin::Side;


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

pub const FIN_REPLY_SWAP_SLTP: u64 = 200;

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
        ExecuteMsg::AddMarket { 
            fin_contract_address, 
            denoms 
        } => {
            let config = CONFIG.load(deps.storage)?;
            if info.sender != config.owner {
                return Err(StdError::generic_err("Unauthorized"));
            }
            FIN_CONTRACTS.save(deps.storage, &fin_contract_address, &denoms)?;
            Ok(Response::new())
        },

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
            
            // Ensure the user sent the correct funds
            if info.funds.len() != 1 {
                return Err(StdError::generic_err("Must send exactly one coin"));
            }
            if side == Side::Base && info.funds[0].denom != denoms.base() {
                return Err(StdError::generic_err("Must send base coin"));
            }
            if side == Side::Quote && info.funds[0].denom != denoms.quote() {
                return Err(StdError::generic_err("Must send quote coin"));
            }
            if info.funds[0].amount != amount {
                return Err(StdError::generic_err("Must send the correct amount"));
            }

            let user_order_key = (
                info.sender.clone(), 
                fin_contract_address.clone(),
                PoolKey::new(side.clone(), price.clone())
            );

            // Check if the user has a previous order at that side and price
            let old_order = USER_ORDERS.may_load(
                deps.storage, 
                user_order_key.clone());
            // For now we only support one order per user per price
            if old_order.is_ok() {
                return Err(StdError::generic_err("User already has an order at that price"));
            }

            // TODO: Allow the user to modify the order amount

            // Save the user order
            USER_ORDERS.save(
                deps.storage, 
                user_order_key.clone(), 
                &UserOrder {amount, price_sl, price_tp}
            )?;

            // Send Submit Order message to FIN
            let execute_msg = WasmMsg::Execute {
                contract_addr: fin_contract_address.to_string(),
                msg: to_json_binary(&rujira_rs::fin::ExecuteMsg::Order(vec![(side, price, amount)]))?,
                funds: info.funds,
            };

            let mut response = Response::new().add_event(
                cosmwasm_std::Event::new("autorujira.autosltp")
                    .add_attribute("action", "place_order")
                    .add_attribute("sender", info.sender.to_string()),
            );
            
            response = response.add_submessage(SubMsg::new(execute_msg));
        
            return Ok(response);          
        }

        ExecuteMsg::ExecuteSlTp { 
            user_address,
            fin_contract_address, 
            side, 
            price,
            claim_amount
        } => {
            // Ensure the contract is valid
            let denoms = FIN_CONTRACTS.load(deps.storage, &fin_contract_address)?;

            // Check if the user has a previous order at that side and price
            let user_order_key = (
                user_address.clone(), 
                fin_contract_address.clone(),
                PoolKey::new(side.clone(), price.clone())
            );
            let user_order = USER_ORDERS.load(
                deps.storage, 
                user_order_key.clone())?;

            let current_price = load_oracle_price(denoms.base(), denoms.quote())?;

            if (user_order.price_sl.is_some() && user_order.price_sl.unwrap() >= current_price) ||
               (user_order.price_tp.is_some() && user_order.price_tp.unwrap() <= current_price) {
                
                // Claim the order
                let msg_claim = rujira_rs::fin::ExecuteMsg::Order(vec![(side.clone(), price.clone(), Uint128::zero())]);
                let execute_msg_claim = WasmMsg::Execute {
                    contract_addr: fin_contract_address.to_string(),
                    msg: to_json_binary(&msg_claim)?,
                    funds: vec![Coin::new(Uint128::zero(), denoms.quote())],
                };

                // NOTE: We're receiving the claiming amount to optimize contract access, however we could add one more roundtrip to the trade contract to get the exect available amount
                let claiming_denom = if side == Side::Base { denoms.quote() } else { denoms.base() };
                let claiming_funds = vec![Coin::new(claim_amount.clone(), claiming_denom)];

                // Swap the funds
                let msg_swap = rujira_rs::fin::ExecuteMsg::Swap {
                    min_return: None,
                    to: None,
                    callback: None,
                };
                let execute_msg_swap = WasmMsg::Execute {
                    contract_addr: fin_contract_address.to_string(),
                    msg: to_json_binary(&msg_swap)?,
                    funds: claiming_funds,
                };
    
                let mut response = Response::new().add_event(
                    cosmwasm_std::Event::new("autorujira.autosltp")
                        .add_attribute("action", "claim_order")
                        .add_attribute("sender", info.sender.to_string()),
                );

                response = response.add_submessage(SubMsg::reply_never(execute_msg_claim));
                response = response.add_submessage(SubMsg::reply_on_success(execute_msg_swap, FIN_REPLY_SWAP_SLTP));

                // Save the user address to be able to send the funds in the reply
                IN_FLIGHT_USER.save(deps.storage, &user_address)?;

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
    Ok(Decimal::one())
}

fn extract_received_tokens(
    env: Env,
    msg: SubMsgResult,
) -> StdResult<Coin> {
    let mut received_coin = Coin::default();

    if let SubMsgResult::Ok(SubMsgResponse { events, .. }) = msg {
        // Find the event that contains the amount of tokens received
        for event in events {
            if event.ty == "coin_received" { // Event must be coin_received
                let mut event_received_coin: Coin = Coin::default();
                let mut event_receiver_ok = false;
                for attr in event.attributes {
                    if attr.key == "receiver" && attr.value == env.contract.address.as_str(){ // The receiver must be this contract
                        event_receiver_ok = true;
                    } else if attr.key == "amount" {
                        event_received_coin = attr.value.parse()?;                        
                    }
                }
                if event_receiver_ok {
                    received_coin = event_received_coin;
                }
            }
        }
    }

    Ok(received_coin)
}

pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> StdResult<Response> {
    match reply.id {
        id if id == FIN_REPLY_SWAP_SLTP => {
            let received_coin = extract_received_tokens(_env, reply.result)?;
            if (received_coin.amount > Uint128::zero()) {
                let user_address = IN_FLIGHT_USER.load(deps.storage)?;
                IN_FLIGHT_USER.save(deps.storage, &Addr::unchecked(""))?;
                // TODO: Calculate fees
                let fees: Uint128 = Uint128::new(1);
                let user_amount = received_coin.amount - fees;
                return Ok(Response::new().add_message(BankMsg::Send {
                    to_address: user_address.to_string(),
                    amount: vec![Coin::new(user_amount, received_coin.denom)],
                }));
            }
            return Err(StdError::generic_err("no received coins".to_string()))
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