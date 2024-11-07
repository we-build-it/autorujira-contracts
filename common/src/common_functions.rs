use anybuf::Anybuf;
use cosmwasm_std::{
    Addr, BalanceResponse, BankQuery, Coin, CosmosMsg, Deps, Env, QueryRequest, StdResult, Uint128,
};

/// Builds an Authz message to execute a contract on behalf of a user.
///
/// # Arguments
///
/// * `env` - The environment information.
/// * `user` - The address of the user on whose behalf the contract will be executed.
/// * `contract_addr` - The address of the contract to execute.
/// * `msg_str` - The message to send to the contract, in JSON string format.
/// * `funds` - The funds to send along with the contract execution.
///
/// # Returns
///
/// * `StdResult<CosmosMsg>` - The constructed Authz message wrapped in a CosmosMsg.
pub fn build_authz_msg(
    env: Env,
    user: Addr,
    contract_addr: Addr,
    msg_str: String,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    // Construct MsgExecuteContract using Anybuf
    let mut execute_contract_buf = Anybuf::new()
        .append_string(1, &user.to_string()) // sender (field 1)
        .append_string(2, &contract_addr.to_string()) // contract (field 2)
        .append_string(3, &msg_str); // msg (field 3)

    // Add funds to the message if provided
    if !funds.is_empty() {
        let funds_bufs: Vec<Anybuf> = funds
            .iter()
            .map(|fund| {
                Anybuf::new()
                    .append_string(1, &fund.denom) // denom (field 1)
                    .append_string(2, &fund.amount.to_string()) // amount (field 2)
            })
            .collect();

        execute_contract_buf = execute_contract_buf.append_repeated_message(5, &funds_bufs);
    }

    let execute_contract_bytes = execute_contract_buf.as_bytes();

    // Wrap MsgExecuteContract in an Any message
    let execute_contract_any_buf = Anybuf::new()
        .append_string(1, "/cosmwasm.wasm.v1.MsgExecuteContract") // type_url (field 1)
        .append_bytes(2, &execute_contract_bytes); // value (field 2)

    // Construct MsgExec using Anybuf
    let msg_exec_buf = Anybuf::new()
        .append_string(1, &env.contract.address.to_string()) // grantee (field 1)
        .append_repeated_message(2, &[execute_contract_any_buf]); // msgs (field 2)

    let cosmos_msg = CosmosMsg::Stargate {
        type_url: "/cosmos.authz.v1beta1.MsgExec".to_string(),
        value: msg_exec_buf.as_bytes().into(),
    };
    Ok(cosmos_msg)
}

pub fn query_token_balance(deps: Deps, address: &Addr, denom: String) -> StdResult<Uint128> {
    let balance_response: BalanceResponse =
        deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
            address: address.to_string(),
            denom,
        }))?;

    Ok(balance_response.amount.amount)
}
