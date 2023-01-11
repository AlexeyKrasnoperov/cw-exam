use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::{
    msg::InstantiateMsg,
    state::{State, OWNER, STATE},
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const HIGHEST_BID_KEY: &str = "highest_bid";
const WINNER_KEY: &str = "winner";

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
    use cosmwasm_std::{Addr, Coin, Deps, StdResult};

    use crate::msg::{AddressBidResp, HighestBidResp, WinnerResp};
    use crate::state::STATE;

    use super::{ATOM, HIGHEST_BID_KEY};

    pub fn highest_bid(deps: Deps) -> StdResult<HighestBidResp> {
        let highest_bid_info = STATE.load(deps.storage, HIGHEST_BID_KEY.to_string())?;

        Ok(HighestBidResp {
            address: highest_bid_info.address,
            bid: highest_bid_info.bid,
        })
    }

    pub fn address_bid(deps: Deps, address: String) -> StdResult<AddressBidResp> {
        let address_bid_info = STATE.may_load(deps.storage, address.to_string())?;

        if address_bid_info.is_some() {
            Ok(AddressBidResp {
                bid: address_bid_info.unwrap().bid,
            })
        } else {
            Ok(AddressBidResp {
                bid: Coin::new(0, ATOM),
            })
        }
    }

    pub fn winner(deps: Deps) -> StdResult<WinnerResp> {
        let winner_info = STATE.may_load(deps.storage, "winner".to_string())?;

        if winner_info.is_some() {
            let winner_info = winner_info.unwrap();
            Ok(WinnerResp {
                address: winner_info.address,
                bid: winner_info.bid,
            })
        } else {
            Ok(WinnerResp {
                address: Addr::unchecked(""),
                bid: Coin::new(0, ATOM),
            })
        }
    }
}

pub mod exec {
    use cosmwasm_std::{BankMsg, Coin, DepsMut, Env, MessageInfo, Response};

    use crate::{
        error::ContractError,
        state::{State, OWNER, STATE},
    };

    use super::{ATOM, HIGHEST_BID_KEY, WINNER_KEY};

    pub fn bid(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        // TODO: Send comission to the owner
        // TODO: Allow highest bidder to increase their bid

        let owner = OWNER.load(deps.storage)?;
        if info.sender == owner {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }

        let winner = STATE.may_load(deps.storage, WINNER_KEY.to_string())?;
        if winner.is_some() {
            return Err(ContractError::BiddingAlreadyClosed {});
        }

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

    pub fn close(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;

        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.into(),
            });
        }

        let winner = STATE.may_load(deps.storage, WINNER_KEY.to_string())?;
        if winner.is_some() {
            return Err(ContractError::BiddingAlreadyClosed {});
        }

        let highest_bid_info = STATE.load(deps.storage, HIGHEST_BID_KEY.to_string())?;

        STATE.save(
            deps.storage,
            WINNER_KEY.to_string(),
            &State {
                address: highest_bid_info.address,
                bid: highest_bid_info.bid.clone(),
            },
        )?;

        let bank_msg = BankMsg::Send {
            to_address: owner.to_string(),
            amount: [highest_bid_info.bid.clone()].to_vec(),
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "close")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn retract(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let winner = STATE.may_load(deps.storage, WINNER_KEY.to_string())?;
        if winner.is_none() {
            return Err(ContractError::BiddingNotClosed {});
        }

        if winner.unwrap().address == info.sender {
            return Err(ContractError::WinnerCannotRetract {});
        }

        let address_bid_info = STATE.may_load(deps.storage, info.sender.to_string())?;

        if address_bid_info.is_some() {
            let bank_msg = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: [address_bid_info.unwrap().bid].to_vec(),
            };

            let resp = Response::new()
                .add_message(bank_msg)
                .add_attribute("action", "retract")
                .add_attribute("sender", info.sender.as_str());

            Ok(resp)
        } else {
            return Err(ContractError::NoBidFound {
                address: info.sender.to_string(),
            });
        }
    }
}
