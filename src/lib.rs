use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use error::ContractError;
use msg::{InstantiateMsg, QueryMsg, ExecMsg};

mod contract;
pub mod error;
pub mod msg;
#[cfg(any(test, feature = "tests"))]
pub mod multitest;
mod state;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::instantiate(deps, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use msg::QueryMsg::*;

    match msg {
        HighestBid {} => to_binary(&contract::query::highest_bid(deps)?),
        AddressBid {address} => to_binary(&contract::query::address_bid(deps, address)?),
        Winner {} => to_binary(&contract::query::winner(deps)?),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use msg::ExecMsg::*;

    match msg {
        Bid {} => contract::exec::bid(deps, env, info).map_err(ContractError::from),
        Close {} => contract::exec::close(deps, env, info).map_err(ContractError::from),
        Retract {} => contract::exec::retract(deps, env, info).map_err(ContractError::from),
    }
}
