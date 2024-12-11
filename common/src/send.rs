use crate::common_functions::{build_authz_msg, AuthzMessageType};
use cosmwasm_std::{Addr, Coin, CosmosMsg, Env, StdResult};

/// Constructs an Authz message to send tokens.
///
/// # Arguments
///
/// * `env` - The environment information.
/// * `user` - The address of the user who will stake the tokens.
/// * `to_address` - The address of target.
/// * `amount` - The amount to send.
/// * `denom` - The denomination of the token to send.
///
/// # Returns
///
/// * `StdResult<CosmosMsg>` - The constructed Authz send message.
pub fn build_send_msg(
    env: Env,
    user: Addr,
    to_address: Addr,
    amount: u128,
    denom: String,
) -> StdResult<CosmosMsg> {
    build_authz_msg(
        env.clone(),
        user.clone(),
        AuthzMessageType::Send {
            to_address,
            amount: vec![Coin {
                denom: denom,
                amount: amount.into(),
            }],
        },
    )
}
