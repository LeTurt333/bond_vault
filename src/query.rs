use crate::state::*;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, Binary, Deps, to_binary, Order};

pub fn get_admin(deps: Deps) -> StdResult<Binary> {
    let storage = CONFIG.load(deps.storage)?;
    to_binary(&AdminResponse {
        admin: storage.admin.into_string(),
    })
}

pub fn get_config(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    to_binary(&ConfigResponse {
        config,
    })
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

pub fn get_user_purchase_infos(deps: Deps, user_wallet: String) -> StdResult<Binary> {

    let user = deps.api.addr_validate(&user_wallet)?;

    let purchase_infos: Vec<(u64, Purchase)> = PURCHASES
        .prefix(user)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&UserPurchaseInfos {
        purchase_infos
    })
}

#[cw_serde]
pub struct AdminResponse {
    pub admin: String,
}
#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
#[cw_serde]
pub struct UserPurchaseIds {
    pub uuids: Vec<u64>
}

#[cw_serde]
pub struct UserPurchaseInfos {
    pub purchase_infos: Vec<(u64, Purchase)>
}
