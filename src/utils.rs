use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;

use cosmwasm_std::Order;
use cosmwasm_std::Coin;
use cosmwasm_std::Uint128;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::{BLOCKS_IN_WEEK, MAX_PURCHASE_AMOUNT};
use crate::error::ContractError;
use crate::msg::*;
use crate::state::*;

//~~~~~~~~~~~~~~~~~~~~~~~~~~
// Checks
//~~~~~~~~~~~~~~~~~~~~~~~~~~

pub fn purchase_funds_check(
    cost_denom: String, 
    funds: &[Coin]
) -> Result<(), ContractError> {

    // assert not empty
    if funds.is_empty() {
        return Err(ContractError::GenericError("Purchase funds empty".to_string()));
    };

    // assert only 1 coin
    if funds.len() != 1 {
        return Err(ContractError::GenericError("More than 1 coin sent".to_string()));
    };

    // assert denom = cost_denom
    if funds[0].denom != cost_denom {
        return Err(ContractError::GenericError("Funds used for purchase wrong denom".to_string()));
    };

    // assert not greater than max purchase amount
    if funds[0].amount > Uint128::from(MAX_PURCHASE_AMOUNT) {
        return Err(ContractError::GenericError("Purchase Amount greater than max purchase amount".to_string()));
    }

    Ok(())
}


pub fn amount_of_purchases_check(
    wallet: Addr,
    deps: Deps
) -> Result<(), ContractError> {

    let user_purchases: Vec<(u64, Purchase)> = PURCHASES
        .prefix(wallet)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()
        .map_err(|_| ContractError::GenericError("Error getting user purchases".to_string()))?;

    if user_purchases.len() >= 5 {
        return Err(ContractError::GenericError("Cannot have more than 5 purchases".to_string()));
    } else {
        return Ok(());
    }
}


pub fn claim_vestable_checks(
    purchase: &Purchase,
    current_block: u64
) -> Result<(), ContractError> {

    // if closed, error
    // if purchase.closed {
    //     return Err(ContractError::GenericError("Cannot claim on a closed purchase".to_string()));
    // }

    // if already_claimed >= amount_purchased, error
    if purchase.already_claimed >= purchase.amount_purchased {
        return Err(ContractError::GenericError("Already fully vested".to_string()));
    }

    // if time.now <= last_claim, error
    if current_block <= purchase.last_claim {
        return Err(ContractError::GenericError("Timing issue".to_string()));
    }

    Ok(())
}



//~~~~~~~~~~~~~~~~~~~~~~~~~~
// Calculations
//~~~~~~~~~~~~~~~~~~~~~~~~~~
pub fn third_dec_ceil(num: Uint128) -> Result<Uint128, ContractError> {
    // add 1000, then divide by 1000, thus ROUNDING UP to the nearest 00_000
    // if fake price is 12_345_678 <12.345678 USDC per JUNO>
    // should be 12_346

    // add 1000 <to round up>
    let add_1000 = num.add(Uint128::from(1_000_u32));

    // divide by 1000, disposing remainder
    let div_1000 = add_1000
        .checked_div(Uint128::from(1_000_u32))
        .map_err(|_| ContractError::GenericError("Round third decimal ceil".to_string()))?;

    // multiply by 1000 so initial size returned
    div_1000
        .checked_mul(Uint128::from(1_000_u32))
        .map_err(|_| ContractError::GenericError("Round third decimal ceil".to_string()))
}

pub fn third_dec_floor(num: Uint128) -> Result<Uint128, ContractError> {
    // divide by 1000, disposing remainder
    let div_1000 = num
        .checked_div(Uint128::from(1_000_u32))
        .map_err(|_| ContractError::GenericError("Round third decimal floor".to_string()))?;

    // multiply by 1000 so initial size returned
    div_1000
        .checked_mul(Uint128::from(1_000_u32))
        .map_err(|_| ContractError::GenericError("Round third decimal ceil".to_string()))
}

// Simply adds discount to Juno Amount
// IE - 10% discount returns Juno Amount * 1.1
pub fn calc_juno_amt(
    discount: u128,
    base_rate: Uint128,
    user_funds: Uint128,
) -> Result<Uint128, ContractError> {
    // price needs to be rounded up
    // funds sent in needs to be rounded down
    // -- Rounding funds down isn't fair to the user <regardless of a discount or not>
    // -- So we manually check that funds_sent_in modulo 1000 == 0, and error if not
    // -- AKA - Users should only buy with an amount rounded to third decimal
    if user_funds % Uint128::from(1000_u32) != Uint128::from(0_u32) {
        return Err(ContractError::GenericError(
            "Amount sent not rounded to 3rd decimal".to_string(),
        ));
    };

    // Round price to 3rd decimal ceiling
    let rounded_price = third_dec_ceil(base_rate)?;

    // Round funds to third decimal floor
    let floored_funds = third_dec_floor(user_funds)?;

    // Juno Amount before Discount bump applied
    let before_discount = Uint128::from(1_000_000_u32)
        .checked_multiply_ratio(floored_funds, rounded_price)
        .map_err(|_| ContractError::GenericError("calc_juno_amt | before_discount".to_string()))?;

    // Juno Amount after Discount bump applied
    let after_discount =
        before_discount.checked_multiply_ratio(100_u128.add(discount), 100_u128).map_err(|_| {
            ContractError::GenericError("calc_juno_amt | before_discount".to_string())
        })?;

    Ok(after_discount)
}


pub fn calc_amt_vested(
    purchase: &Purchase,
    current_block: u64
) -> Result<(Uint128, bool), ContractError> {

    // amount_purchased - already_claimed = amount_left_to_claim
    let amount_left = purchase.amount_purchased
        .checked_sub(purchase.already_claimed)
        .map_err(|_| ContractError::GenericError("overflow on amount_purchased - already_claimed error".to_string()))?;

    if current_block >= purchase.vest_expiration {
        // Shouldn't ever be zero 
        if amount_left.is_zero() {
            return Err(ContractError::GenericError("None left to be vested".to_string()));
        } else {
            return Ok((amount_left, true));
        }
    };

    // use vest_period & amount_purchased to determine amount vested_per_block
    // amount vested per block will be 
    // vest period in weeks * blocks per week = total_blocks
    // amount_purchased / total_blocks = vest_per_block
    let total_blocks = Uint128::from(BLOCKS_IN_WEEK)
        .checked_mul(Uint128::from(purchase.vest_period))
        .map_err(|_| ContractError::GenericError("Overflow on blocks_in_week * purchase.vest_period".to_string()))?;

    let vest_per_block = purchase.amount_purchased
        .checked_div(total_blocks)
        .map_err(|_| ContractError::GenericError("Overflow on amount_purchased / total_blocks".to_string()))?;

    // time.now - last_claim = amount of blocks to vest for
    let blocks_to_claim_for = match current_block.saturating_sub(purchase.last_claim) {
        0 => Err(ContractError::GenericError("Current block <= last claim".to_string())),
        x => Ok(x)
    }?;

    // use vested_per_block & amount_blocks_to_vest_for to calculate JUNO to sent to user
    let claim_amount = Uint128::from(blocks_to_claim_for)
        .checked_mul(vest_per_block)
        .map_err(|_| ContractError::GenericError("Overflow on vest_per_block * blocks_to_claim_for".to_string()))?;


    Ok((claim_amount, false))

}



