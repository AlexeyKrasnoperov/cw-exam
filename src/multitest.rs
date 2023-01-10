use cosmwasm_std::{Addr, Coin, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};

use crate::{
    error::ContractError,
    msg::{AddressBidResp, ExecMsg, HighestBidResp, InstantiateMsg, QueryMsg, WinnerResp},
};
use crate::{execute, instantiate, query};

#[cfg(test)]
mod tests;

pub struct BiddingContract(Addr);

impl BiddingContract {
    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        admin: Option<&Addr>,
        label: &str,
    ) -> StdResult<BiddingContract> {
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg { owner: None },
            &[],
            label,
            admin.map(Addr::to_string),
        )
        .map_err(|err| err.downcast().unwrap())
        .map(BiddingContract)
    }

    pub fn addr(&self) -> &Addr {
        &self.0
    }

    #[track_caller]
    pub fn bid(&self, app: &mut App, sender: &Addr, funds: &[Coin]) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Bid {}, funds)
            .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    #[track_caller]
    pub fn close(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Close {}, &[])
            .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    pub fn retract(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Retract {}, &[])
            .map_err(|err| err.downcast::<ContractError>().unwrap())?;

        Ok(())
    }

    #[track_caller]
    pub fn query_highest_bid(&self, app: &App) -> StdResult<HighestBidResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::HighestBid {})
    }

    #[track_caller]
    pub fn query_address_bid(&self, app: &App, address: String) -> StdResult<AddressBidResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::AddressBid { address: address })
    }

    #[track_caller]
    pub fn query_winner(&self, app: &App) -> StdResult<WinnerResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Winner {})
    }
}
