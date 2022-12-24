use crate::{state::*, ContractError::*, query::*};
use cosmwasm_schema::{cw_serde, QueryResponses};
//use cosmwasm_std::{to_binary, Binary, Deps, StdResult, Uint128, Order};

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Instantiate
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub beingsold_denom: String,
    pub cost_denom: String,
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Execute
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
pub enum ExecuteMsg {
    PausePurchasing {},
    AddSaleFunds {},
    ResumePurchasing {},
    Purchase {vesting_period: u128},
    ClaimAvailable {uuid: u64},
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Query
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    GetAdmin {},
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(UserPurchaseIds)]
    GetUserPurchaseIds {user_wallet: String},
    #[returns(UserPurchaseInfos)]
    GetUserPurchaseInfos { user_wallet: String} 
}