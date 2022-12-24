#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Uint128, Coin};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::{
    msg::*, query::*, state::*, utils::*, 
    MAX_VEST, FAKE_PRICE, MAX_PURCHASE_AMOUNT, BLOCKS_IN_WEEK
};

const CONTRACT_NAME: &str = "crates.io:cpbond";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    PURCHASE_COUNT.save(deps.storage, &1_u64)?;

    PAUSED.save(deps.storage, &true)?;

    BALANCE.save(deps.storage, &Uint128::from(0_u128))?;

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

        //~~~~~~~~~~~~~~
        // Admin Only
        //~~~~~~~~~~~~~~
        ExecuteMsg::PausePurchasing {} => pause_purchasing(deps, env, &info.sender),
        ExecuteMsg::ResumePurchasing {} => resume_purchasing(deps, env, &info.sender),
        ExecuteMsg::AddSaleFunds {} => add_sale_funds(deps, env, &info.sender, &info.funds),

        //~~~~~~~~~~~~~~
        // User
        //~~~~~~~~~~~~~~
        ExecuteMsg::Purchase {vesting_period} => {
            if PAUSED.load(deps.storage)? {
                Err(ContractError::GenericError("Purchasing is paused".to_string()))
            } else {
                execute_purchase(deps, env, info.sender, &info.funds, vesting_period)
            }
        },
        ExecuteMsg::ClaimAvailable {uuid} => claim_available(deps, env, info.sender, uuid),
    }
}

pub fn pause_purchasing(
    deps: DepsMut, 
    env: Env, 
    sender: &Addr
) -> Result<Response, ContractError> {

    let config: Config = CONFIG.load(deps.storage)?;

    if sender != &config.admin {
        return Err(ContractError::Unauthorized);
    } else {
        PAUSED.update(
            deps.storage,
            |x| -> StdResult<bool> {
                Ok(true)
            }
        )?;
    };


    Ok(Response::default())

}

pub fn resume_purchasing(
    deps: DepsMut, 
    env: Env, 
    sender: &Addr
) -> Result<Response, ContractError> {

    let config: Config = CONFIG.load(deps.storage)?;

    if sender != &config.admin {
        return Err(ContractError::Unauthorized);
    } else {
        PAUSED.update(
            deps.storage,
            |x| -> StdResult<bool> {
                Ok(false)
            }
        )?;
    };


    Ok(Response::default())

}

pub fn add_sale_funds(
    deps: DepsMut, 
    env: Env, 
    sender: &Addr,
    funds: &[Coin]
) -> Result<Response, ContractError> {

    let config: Config = CONFIG.load(deps.storage)?;

    // if sender != &config.admin {
    //     return Err(ContractError::Unauthorized);
    // };

    // assert not empty
    if funds.is_empty() {
        return Err(ContractError::GenericError("Purchase funds empty".to_string()));
    };

    // assert only 1 coin
    if funds.len() != 1 {
        return Err(ContractError::GenericError("More than 1 coin sent".to_string()));
    };

    // assert denom = cost_denom
    if funds[0].denom != config.cost_denom {
        return Err(ContractError::GenericError("Funds used for purchase wrong denom".to_string()));
    };


    // Checks pass, update balance
    BALANCE.update(
        deps.storage, 
        |o| -> StdResult<Uint128> {
            o.checked_add(funds[0].amount).map_err(|e| e.into())
        }
    )?;

    Ok(Response::default())
}

pub fn execute_purchase(
    deps: DepsMut,
    env: Env,
    user_wallet: Addr,
    user_funds: &[Coin],
    vesting_period: u128,
) -> Result<Response, ContractError> {

    if vesting_period > MAX_VEST {
        return Err(ContractError::GenericError("Max vesting period is 52 weeks".to_string()));
    }

    // Check that user doesn't already have 5 purchases
    amount_of_purchases_check(user_wallet.clone(), deps.as_ref())?;

    // CHECK: denom = cost_denom | not empty | only 1 coin | not greater than max purchase amount
    let config: Config = CONFIG.load(deps.storage)?;
    purchase_funds_check(config.cost_denom, user_funds)?;

    // fully vested block height = vesting_period * blocks_in_week + current_block_height
    let vesting_blocks = vesting_period
        .checked_mul(BLOCKS_IN_WEEK)
        .ok_or_else(|| ContractError::GenericError("vesting_blocks overflow".to_string()))?;

    let fully_vested_blockheight = vesting_blocks
        .checked_add(env.block.height.into())
        .ok_or_else(|| ContractError::GenericError("fully_vested_blockheight overflow".to_string()))?;

    //~~~~~
    //~~~~~
    // This is where a query to price oracle would happen, FAKE_PRICE used for time being
    //~~~~~
    //~~~~~

    // Calculate amount being purchased using vesting_period for discount, price, and amount of funds sent in
    let juno_to_user = calc_juno_amt(vesting_period, FAKE_PRICE, user_funds[0].amount)?;

    // Make sure contract has enough JUNO to complete purchase
    let contract_balance = BALANCE.load(deps.storage)?;

    if juno_to_user >= contract_balance {
        return Err(
            ContractError::GenericError(
                format!("Not enough JUNO to complete sale | JUNO: {} | juno_to_user: {}", contract_balance, juno_to_user)
            ));
    }

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

    // Save new purchase
    PURCHASES
        .save(deps.storage, (user_wallet, uuid), &purchase)
        .map_err(|_| ContractError::GenericError("Save Purchase Error".to_string()))?;

    // Update contract available balance
    BALANCE
        .update(deps.storage, |o| -> StdResult<Uint128> {
            o.checked_sub(juno_to_user).map_err(|e| e.into())
        })?;

    // Update purchase count
    PURCHASE_COUNT
        .update(deps.storage, |old| -> Result<u64, ContractError> { // initialized on init
            if old >= u64::MAX - 1_u64 {
                Ok(1_u64)
            } else {
                Ok(old + 1_u64)
            }
        })
        .map_err(|_| ContractError::GenericError("Update purchase count error".to_string()))?;

    // Decide whether funds from user stay in this contract or go elsewhere like community pool
    Ok(Response::new().add_attribute("Call", "Purchase"))
}

pub fn claim_available(
    deps: DepsMut,
    env: Env,
    user_wallet: Addr,
    uuid: u64
) -> Result<Response, ContractError> {

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

    Ok(Response::new().add_attribute("Call", "Claim vestable"))
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Query
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAdmin {} => to_binary(&get_admin(deps)?),
        QueryMsg::GetConfig {} => to_binary(&get_config(deps)?),
        QueryMsg::GetUserPurchaseIds { user_wallet } => to_binary(&get_user_purchase_ids(deps, user_wallet)?),
        QueryMsg::GetUserPurchaseInfos { user_wallet } => to_binary(&get_user_purchase_infos(deps, user_wallet)?)
    }
}