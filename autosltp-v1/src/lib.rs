pub mod msg;
pub mod state;
pub mod contract;
pub mod fin;
pub mod event_utils;

use cosmwasm_std::{
    entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Binary, Reply
};

use crate::msg::{
  InstantiateMsg, ExecuteMsg,
};

#[entry_point]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg)
  -> StdResult<Response> {
    contract::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg)
  -> StdResult<Response> {
    contract::execute(deps, env, info, msg)
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) 
  -> StdResult<Response> {
    contract::reply(deps, env, reply)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: msg::QueryMsg)
  -> StdResult<Binary>
{
    contract::query(deps, env, msg)
}
