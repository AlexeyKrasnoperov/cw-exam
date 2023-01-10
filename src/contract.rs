use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult};
use cw2::{set_contract_version};

use crate::{
    msg::InstantiateMsg,
    state::{State, OWNER, STATE},
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const HIGHEST_BID_KEY : &str = "highest_bid";

const ATOM: &str = "atom";

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(
        deps.storage,
        HIGHEST_BID_KEY.to_string(),
        &State {
            address: info.sender.clone(),
            bid: Coin::new(0, ATOM),
        },
    )?;

    if msg.owner.is_some() {
        OWNER.save(deps.storage, &msg.owner.unwrap())?;
    } else {
        OWNER.save(deps.storage, &info.sender)?;
    }

    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::msg::{AddressBidResp, HighestBidResp, WinnerResp};
    use crate::state::STATE;

    use super::HIGHEST_BID_KEY;

    pub fn highest_bid(deps: Deps) -> StdResult<HighestBidResp> {
        let highest_bid_info = STATE.load(deps.storage, HIGHEST_BID_KEY.to_string())?;

        Ok(HighestBidResp {
            address: highest_bid_info.address,
            bid: highest_bid_info.bid,
        })
    }

    pub fn address_bid(deps: Deps, address: String) -> StdResult<AddressBidResp> {
        let address_bid_info = STATE.load(deps.storage, address.to_string())?;
        Ok(AddressBidResp {
            bid: address_bid_info.bid,
        })
    }

    pub fn winner(deps: Deps) -> StdResult<WinnerResp> {
        let winner_info = STATE.load(deps.storage, "winner".to_string())?;

        Ok(WinnerResp {
            address: winner_info.address,
            bid: winner_info.bid,
        })
    }
}

pub mod exec {
    use cosmwasm_std::{
        BankMsg, DepsMut, Env, MessageInfo, Response, Coin,
    };

    use crate::{
        error::ContractError,
        state::{OWNER, STATE, State},
    };

    use super::{ATOM, HIGHEST_BID_KEY};

    pub fn bid(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // TODO: Raise error in case if bidder == owner
        // TODO: Send comission to the owner

        let highest_bid_info = STATE.load(deps.storage, HIGHEST_BID_KEY.to_string())?;
        let mut resp = Response::default();

        let native_coin_bid = info.funds.iter().find(|coin| coin.denom == ATOM);

        if native_coin_bid.is_some() {
            let address_bid_info = STATE.may_load(deps.storage, info.sender.to_string())?;
            let mut total_address_bid = native_coin_bid.unwrap().amount;
            if address_bid_info.is_some() {
                total_address_bid += address_bid_info.unwrap().bid.amount
            }

            if total_address_bid > highest_bid_info.bid.amount {
                STATE.save(
                    deps.storage,
                    info.sender.to_string(),
                    &State {
                        address: info.sender.clone(),
                        bid: Coin::new(total_address_bid.u128(), ATOM),
                    },
                )?;
            
                STATE.save(
                    deps.storage,
                    HIGHEST_BID_KEY.to_string(),
                    &State {
                        address: info.sender.clone(),
                        bid: Coin::new(total_address_bid.u128(), ATOM),
                    },
                )?;
            } else {
                return Err(ContractError::InsufficientBid {
                    bid: total_address_bid.to_string(),
                    highest_bid: highest_bid_info.bid.amount.to_string(),
                }
                .into());
            }
        } else {
            return Err(ContractError::IncorrectBid {}.into());
        }

        resp = resp
            .add_attribute("action", "bid")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute(HIGHEST_BID_KEY, highest_bid_info.bid.to_string());

        Ok(resp)
    }

    pub fn close(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;

        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }

        let funds = deps.querier.query_all_balances(env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: funds,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "close")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn retract(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;

        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }

        let funds = deps.querier.query_all_balances(env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: funds,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "retract")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }
}
