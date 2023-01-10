use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized - only {owner} can call it")]
    Unauthorized { owner: String },

    #[error("Insufficient Bid - the bid {bid} is lower than the highest bid {highest_bid}")]
    InsufficientBid { bid: String, highest_bid: String },

    #[error("Incorrect Bid - the bid should be done using the native token")]
    IncorrectBid {},
}
