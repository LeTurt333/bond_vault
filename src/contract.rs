use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Order;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::Uint128;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::*;
use crate::state::*;
use crate::utils::*;

const CONTRACT_NAME: &str = "crates.io:cpbond";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const FAKE_PRICE: Uint128 = Uint128::new(12_345_678_u128);

// 10,000 USDC, any arbitrary value, can be changed by putting in Config instead of const
const MAX_PURCHASE_AMOUNT: u128 = 10_000_000_000_u128;

const MAX_VEST: u128 = 52_u128;
const BLOCKS_IN_WEEK: u128 = 100_000_u128;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Instantiate
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let validated_admin = match msg.admin {
        Some(adm) => deps.api.addr_validate(&adm)?,
        None => info.sender,
    };

    CONFIG.save(
        deps.storage,
        &Config {
            admin: validated_admin,
            beingsold_denom: msg.beingsold_denom,
            cost_denom: msg.cost_denom,
        },
    )?;

    PURCHASE_COUNT.save(
        deps.storage,
        &1_u64
    )?;

    Ok(Response::new().add_attribute("Called", "Instantiate"))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Execute
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Purchase {vesting_period} => {

            if vesting_period > MAX_VEST {
                return Err(ContractError::GenericError("Max vesting period is 52 weeks".to_string()));
            }

            // Check that user doesn't already have 5 purchases
            amount_of_purchases_check(info.sender.clone(), deps.as_ref())?;

            // CHECK: denom = cost_denom | not empty | only 1 coin | not greater than max purchase amount
            let config: Config = CONFIG.load(deps.storage)?;
            purchase_funds_check(config.cost_denom, &info.funds)?;

            execute_purchase(deps, env, info.sender, info.funds[0].amount, vesting_period)
        }

        ExecuteMsg::ClaimAvailable {} => Ok(Response::default()),
    }
}

pub fn execute_purchase(
    deps: DepsMut,
    env: Env,
    user_wallet: Addr,
    user_funds_amount: Uint128,
    vesting_period: u128,
) -> Result<Response, ContractError> {
    let discount = vesting_period;

    let fully_vested_blockheight = vesting_period
        .checked_mul(BLOCKS_IN_WEEK)
        .ok_or_else(|| ContractError::GenericError("Fully vested overflow".to_string()))?;

    // Get price from oracle, FAKE_PRICE used for now
    // let fake_price = query_oracle_function()

    let juno_to_user = calc_juno_amt(discount, FAKE_PRICE, user_funds_amount)?;

    // Create Purchase Item in state with user_address, UUID, amount, and purchase object
    let uuid = PURCHASE_COUNT.load(deps.storage)?;

    let purchase: Purchase = Purchase {
        vest_period: vesting_period.try_into().map_err(|_| ContractError::ToDo)?,
        vest_expiration: fully_vested_blockheight.try_into().map_err(|_| ContractError::ToDo)?,
        amount_purchased: juno_to_user,
        already_claimed: Uint128::from(0_u128),
        last_claim: env.block.height,
        //closed: false,
    };

    PURCHASES
        .save(deps.storage, (user_wallet, uuid), &purchase)
        .map_err(|_| ContractError::GenericError("Save Purchase Error".to_string()))?;

    PURCHASE_COUNT
        .update(deps.storage, |old| -> Result<u64, ContractError> { // initialized on init
            if old >= u64::MAX - 1_u64 {
                Ok(1_u64)
            } else {
                Ok(old + 1_u64)
            }
        })
        .map_err(|_| ContractError::GenericError("Update purchase count error".to_string()))?;

    // Funds stay in contract or go somewhere?
    Ok(Response::new().add_attribute("Call", "Purchase"))
}

pub fn claim_available(
    deps: DepsMut,
    env: Env,
    user_wallet: Addr,
    uuid: u64
) -> Result<Response, ContractError> {

    // Ok nevermind, not gonna do claim_vestable on all user purchases in 1 tx
    // as if any fail then they all fail
    // handle this manually in a loop on the client with a query that gets all user purchases

    let purchase: Purchase = PURCHASES.load(deps.storage, (user_wallet.clone(), uuid))?;

    // First - Do checks
    claim_vestable_checks(&purchase, env.block.height)?;

    // Second - Calculate amount to be vested & sent
    let (amount_vestable, close_purchase) = calc_amt_vested(&purchase, env.block.height)?;

    if close_purchase {
        PURCHASES.remove(
            deps.storage,
            (user_wallet.clone(), uuid)
        );
    } else {
        PURCHASES.update(
            deps.storage,
            (user_wallet.clone(), uuid),
            |old| -> StdResult<Purchase> {
                let Some(oldx) = old else {
                    return Err(StdError::NotFound{kind: "purchase".to_string()});
                };
                Ok(Purchase {
                    already_claimed: oldx.already_claimed.checked_add(amount_vestable)?,
                    last_claim: env.block.height,
                    //closed: close_purchase,
                    ..oldx
                })
            }
        )?;
    }


    // PURCHASES.update(
    //     deps.storage,
    //     (user_wallet.clone(), uuid),
    //     |old| -> StdResult<Purchase> {
    //         let Some(oldx) = old else {
    //             return Err(StdError::NotFound{kind: "purchase".to_string()});
    //         };
    //         Ok(Purchase {
    //             already_claimed: oldx.already_claimed.checked_add(amount_vestable)?,
    //             last_claim: env.block.height,
    //             //closed: close_purchase,
    //             ..oldx
    //         })
    //     }
    // )?;



    // update last_claim to time.now

    // update already_claimed += amount_sent_to_user




    Ok(Response::default())
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Query
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAdmin {} => to_binary(&get_admin(deps)?),
        QueryMsg::GetConfig {} => to_binary(&get_config(deps)?),
        QueryMsg::GetUserPurchaseIds { user_wallet } => to_binary(&get_user_purchase_ids(deps, user_wallet)?)
    }
}





pub fn execute_claim_available(
    deps: DepsMut,
    env: Env,
    user_wallet: Addr,
) -> Result<Response, ContractError> {

    // first, get all PURCHASES by the user
    // Should be no more than 5
    let user_purchases: Vec<(u64, Purchase)> = PURCHASES
        .prefix(user_wallet.clone())
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()
        .map_err(|_| ContractError::GenericError("Error getting user purchases".to_string()))?;


    // Do checks on each Purchase in user_purchases

    // Ok nevermind, not gonna do claim_vestable on all user purchases in 1 tx
    // as if any fail then they all fail
    // handle this manually in a loop on the client with a query that gets all user purchases




    //~~~~~~ claim vestable ~~~~~~~~~~//

    // CHECKS //

    // if closed, error

    // if already_claimed >= amount_purchased, error

    // if time.now <= last_claim, error

    // CALCULATION //

    // amount_purchased - already_claimed = amount_left_to_claim

    // use vest_period & amount_purchased?? to determine amount vested_per_block

    // time.now - last_claim = amount of blocks to vest for

    // use vested_per_block & amount_blocks_to_vest_for to calculate JUNO to sent to user

    // UPDATE //

    // update last_claim to time.now

    // update already_claimed += amount_sent_to_user




    Ok(Response::default())
}

