use crate::{state::*, ContractError};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Binary, Deps, StdResult, Uint128, Order};

//~~~~~~~~~~~~~~~
// Instantiate
//~~~~~~~~~~~~~~~
#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub beingsold_denom: String,
    pub cost_denom: String,
}

//~~~~~~~~~~~~~~~
// Execute
//~~~~~~~~~~~~~~~
#[cw_serde]
pub enum ExecuteMsg {
    //Add { amount: u8, name: String },

    // Purchase { choice_of_vesting_period : 1 week or 2 weeks }
    Purchase {
        vesting_period: u128,
    },

    // ClaimAvailable {}
    ClaimAvailable {},
}

#[cw_serde]
pub enum VestingChoice {
    OneWeek,
    TwoWeeks,
}

// what if instead of an option of 1 week / 2 weeks
// it was X weeks with a max of 52
// and that was the discount they got

// 1 week vest = 1% discount
// 23 week vest = 23% discount

//~~~~~~~~~~~~~~~
// Query
//~~~~~~~~~~~~~~~
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminResponse)]
    GetAdmin {},
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(UserPurchaseIds)]
    GetUserPurchaseIds {user_wallet: String}
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: String,
}

pub fn get_admin(deps: Deps) -> StdResult<Binary> {
    let storage = CONFIG.load(deps.storage)?;
    to_binary(&AdminResponse {
        admin: storage.admin.into_string(),
    })
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

pub fn get_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&ConfigResponse {
        config,
    })
}


#[cw_serde]
pub struct UserPurchaseIds {
    pub uuids: Vec<u64>
}




pub fn get_user_purchase_ids(deps: Deps, user_wallet: String) -> StdResult<Binary> {

    let user = deps.api.addr_validate(&user_wallet)?;

    let user_purchases: Vec<(u64, Purchase)> = PURCHASES
        .prefix(user)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    let ids = user_purchases
        .iter()
        .map(|p| p.0)
        .collect::<Vec<u64>>();

    to_binary(&UserPurchaseIds {
        uuids: ids
    })

}
