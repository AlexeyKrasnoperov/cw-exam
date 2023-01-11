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

const COMMISSION: f64 = 0.10;

const ATOM: &str = "atom";

pub fn instantiate(deps: DepsMut, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(
        deps.storage,
        HIGHEST_BID_KEY.to_string(),
        &State {
            address: info.sender.clone(),
            bid: Coin::new(0, ATOM),
            commission: Coin::new(0, ATOM),
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
    use cosmwasm_std::{BankMsg, Coin, DepsMut, Env, MessageInfo, Response, Uint128};

    use crate::{
        error::ContractError,
        state::{State, OWNER, STATE},
    };

    use super::{ATOM, COMMISSION, HIGHEST_BID_KEY, WINNER_KEY};

    pub fn bid(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender == owner {
            return Err(ContractError::OwnerCannotBid {});
        }

        let winner = STATE.may_load(deps.storage, WINNER_KEY.to_string())?;
        if winner.is_some() {
            return Err(ContractError::BiddingAlreadyClosed {});
        }

        let highest_bid_info = STATE.load(deps.storage, HIGHEST_BID_KEY.to_string())?;
        let mut resp = Response::default();

        let native_coin_bid = info.funds.iter().find(|coin| coin.denom == ATOM);

        if native_coin_bid.is_some() {
            let native_coin_bid = native_coin_bid.unwrap();
            let address_bid_info = STATE.may_load(deps.storage, info.sender.to_string())?;
            let mut total_address_bid = native_coin_bid.amount;
            if address_bid_info.is_some() {
                total_address_bid += address_bid_info.clone().unwrap().bid.amount
            }

            if total_address_bid > highest_bid_info.bid.amount {
                let commission = Coin::new(
                    (native_coin_bid.amount.u128() as f64 * COMMISSION) as u128,
                    native_coin_bid.clone().denom,
                );

                let mut total_commission = commission.amount;
                if address_bid_info.is_some() {
                    total_commission += address_bid_info.unwrap().commission.amount
                }

                STATE.save(
                    deps.storage,
                    info.sender.to_string(),
                    &State {
                        address: info.sender.clone(),
                        bid: Coin::new(total_address_bid.u128(), ATOM),
                        commission: Coin::new(total_commission.u128(), ATOM),
                    },
                )?;

                STATE.save(
                    deps.storage,
                    HIGHEST_BID_KEY.to_string(),
                    &State {
                        address: info.sender.clone(),
                        bid: Coin::new(total_address_bid.u128(), ATOM),
                        commission: Coin::new(total_commission.u128(), ATOM),
                    },
                )?;

                resp = resp
                    .add_attribute("action", "bid")
                    .add_attribute("sender", info.sender.as_str())
                    .add_attribute(HIGHEST_BID_KEY, highest_bid_info.bid.to_string());

                if commission.amount > Uint128::new(0) {
                    let bank_msg = BankMsg::Send {
                        to_address: owner.to_string(),
                        amount: [commission.clone()].to_vec(),
                    };

                    Ok(resp.add_message(bank_msg))
                } else {
                    Ok(resp)
                }
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
                commission: highest_bid_info.commission.clone(),
            },
        )?;

        let resp = Response::new()
            .add_attribute("action", "close")
            .add_attribute("sender", info.sender.as_str());

        if highest_bid_info.bid.amount > Uint128::new(0) {
            let to_be_paid = Coin::new(
                highest_bid_info.bid.amount.u128() - highest_bid_info.commission.amount.u128(),
                highest_bid_info.bid.denom,
            );
            let bank_msg = BankMsg::Send {
                to_address: owner.to_string(),
                amount: [to_be_paid.clone()].to_vec(),
            };
            Ok(resp.add_message(bank_msg))
        } else {
            Ok(resp)
        }
    }

    pub fn retract(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        receiver: Option<String>,
    ) -> Result<Response, ContractError> {
        let winner = STATE.may_load(deps.storage, WINNER_KEY.to_string())?;
        if winner.is_none() {
            return Err(ContractError::BiddingNotClosed {});
        }

        if winner.unwrap().address == info.sender {
            return Err(ContractError::WinnerCannotRetract {});
        }

        let address_bid_info = STATE.may_load(deps.storage, info.sender.to_string())?;

        if address_bid_info.is_some() {
            let address_bid_info = address_bid_info.unwrap();
            let receiver = receiver.unwrap_or(info.sender.to_string());
            let to_be_returned = Coin::new(
                address_bid_info.bid.amount.u128() - address_bid_info.commission.amount.u128(),
                address_bid_info.bid.denom,
            );

            let bank_msg = BankMsg::Send {
                to_address: receiver,
                amount: [to_be_returned].to_vec(),
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
