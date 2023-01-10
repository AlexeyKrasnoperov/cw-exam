use cosmwasm_std::{Addr, Coin, Empty, coins};
use cw_multi_test::{App, Contract, ContractWrapper};

use crate::{
    execute, instantiate,
    multitest::BiddingContract,
    query, error::ContractError,
};

fn bidding_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}

const ATOM: &str = "atom";

#[test]
fn query_highest_bid() {
    let mut app = App::default();
    let sender = Addr::unchecked("sender");

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        None,
        "Bidding Contract",
        Coin::new(10, ATOM),
    )
    .unwrap();

    let resp = contract.query_highest_bid(&app).unwrap();

    assert_eq!(resp.bid, Coin::new(10, ATOM));
}

#[test]
fn zero_bid() {
    let mut app = App::default();

    let contract_id = app.store_code(bidding_contract());

    let sender = Addr::unchecked("sender");

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        None,
        "Bidding Contract",
        Coin::new(10, ATOM),
    )
    .unwrap();

    let err = contract.bid(&mut app, &sender, &[]).unwrap_err();

    assert_eq!(
        err,
        ContractError::InsufficientBid { bid: String::from("0"), highest_bid: String::from("10") }
    );
}

#[test]
fn low_bid() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &sender, coins(9, ATOM))
            .unwrap();
    });

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
        Coin::new(10, ATOM),
    )
    .unwrap();

    let err = contract.bid(&mut app, &sender, &[Coin::new(9, ATOM)]).unwrap_err();

    assert_eq!(
        err,
        ContractError::InsufficientBid { bid: String::from("9"), highest_bid: String::from("10") }
    );
}
