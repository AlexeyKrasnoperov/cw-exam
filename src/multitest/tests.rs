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
fn query_highest_bid_no_bids() {
    let mut app = App::default();
    let owner = Addr::unchecked("owner");

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
    )
    .unwrap();

    let resp = contract.query_highest_bid(&app).unwrap();

    assert_eq!(resp.address, owner.to_string());
    assert_eq!(resp.bid, Coin::new(0, ATOM));
}

#[test]
fn query_address_bid_no_bids() {
    let mut app = App::default();
    let owner = Addr::unchecked("owner");

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
    )
    .unwrap();

    let resp = contract.query_address_bid(&app, owner.to_string()).unwrap();

    assert_eq!(resp.bid, Coin::new(0, ATOM));
}

#[test]
fn query_winner_no_bids() {
    let mut app = App::default();
    let owner = Addr::unchecked("owner");

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
    )
    .unwrap();

    let resp = contract.query_winner(&app).unwrap();

    assert_eq!(resp.address, "");
    assert_eq!(resp.bid, Coin::new(0, ATOM));
}

#[test]
fn zero_bid() {
    let mut app = App::default();

    let contract_id = app.store_code(bidding_contract());

    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
    )
    .unwrap();

    let err = contract.bid(&mut app, &sender, &[]).unwrap_err();

    assert_eq!(
        err,
        ContractError::IncorrectBid {}
    );
}

#[test]
fn low_bid() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &sender2, coins(10, ATOM))
            .unwrap();
    });

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
    )
    .unwrap();

    contract.bid(&mut app, &sender1, &[Coin::new(10, ATOM)]).unwrap();
    let err = contract.bid(&mut app, &sender2, &[Coin::new(10, ATOM)]).unwrap_err();

    assert_eq!(
        err,
        ContractError::InsufficientBid { bid: String::from("10"), highest_bid: String::from("10") }
    );
}


#[test]
fn bid() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
    )
    .unwrap();

    contract.bid(&mut app, &sender, &[Coin::new(10, ATOM)]).unwrap();

    let resp = contract.query_highest_bid(&app).unwrap();

    assert_eq!(resp.bid, Coin::new(10, ATOM));
    assert_eq!(resp.address, sender);
}

#[test]
fn successful_bid_after_losing() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &sender2, coins(10, ATOM))
            .unwrap();
    });

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
    )
    .unwrap();

    contract.bid(&mut app, &sender1, &[Coin::new(5, ATOM)]).unwrap();
    contract.bid(&mut app, &sender2, &[Coin::new(6, ATOM)]).unwrap();
    contract.bid(&mut app, &sender1, &[Coin::new(2, ATOM)]).unwrap();

    let resp = contract.query_highest_bid(&app).unwrap();

    assert_eq!(resp.bid, Coin::new(7, ATOM));
    assert_eq!(resp.address, sender1);
}

#[test]
fn unsuccessful_bid_after_losing() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(10, ATOM))
            .unwrap();

        router
            .bank
            .init_balance(storage, &sender2, coins(10, ATOM))
            .unwrap();
    });

    let contract_id = app.store_code(bidding_contract());

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &owner,
        None,
        "Bidding Contract",
    )
    .unwrap();

    contract.bid(&mut app, &sender1, &[Coin::new(5, ATOM)]).unwrap();
    contract.bid(&mut app, &sender2, &[Coin::new(6, ATOM)]).unwrap();
    let err = contract.bid(&mut app, &sender1, &[Coin::new(1, ATOM)]).unwrap_err();

    assert_eq!(
        err,
        ContractError::InsufficientBid { bid: String::from("6"), highest_bid: String::from("6") }
    );

    let resp = contract.query_highest_bid(&app).unwrap();

    assert_eq!(resp.bid, Coin::new(6, ATOM));
    assert_eq!(resp.address, sender2);
}
