use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Addr};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<Addr>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(HighestBidResp)]
    HighestBid {},
    #[returns(AddressBidResp)]
    AddressBid { address: String },
    #[returns(WinnerResp)]
    Winner {},
}

#[cw_serde]
pub enum ExecMsg {
    Bid{},
    Close{},
    Retract{},
}

#[cw_serde]
pub struct HighestBidResp {
    pub address: Addr,
    pub bid: Coin,
}

#[cw_serde]
pub struct AddressBidResp {
    pub bid: Coin,
}

#[cw_serde]
pub struct WinnerResp {
    pub address: Addr,
    pub bid: Coin,
}
