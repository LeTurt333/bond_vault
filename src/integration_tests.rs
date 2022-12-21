#![cfg(test)]
use anyhow::ensure;
use core::fmt::Display;
use std::ops::Add;

use cosmwasm_std::{coins, to_binary, Addr, Binary, Coin, Empty, Uint128}; //BlockInfo};
                                                                          // use cw20::{Cw20Coin, Cw20CoinVerified, Cw20Contract};
                                                                          // use cw_multi_test::{App, Contract, ContractWrapper, Executor};

// use self::create_contract::*;
// use self::create_users::*;
//use self::init_contracts::init_all_contracts;
use crate::{msg::*, state::*};

// const VALID_NATIVE: &str = "ujunox";
// const INVALID_NATIVE: &str = "poopcoin";
// const INVALID_CW20: &str = "juno1as9d8falsjfs98a08u2390uas0f87dasdf98a79s8df7a89asdf987asd8";

pub fn here(ctx: impl Display, line: impl Display, col: impl Display) -> String {
    format!(
        "~~~~~~~~~~~~~~~~~~~ \n \n {} \n line {} | column {} \n ________________________",
        ctx, line, col
    )
}

#[test]
fn test_one() -> Result<(), anyhow::Error> {
    //use std::borrow::BorrowMut;
    use crate::utils::*;
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    use std::ops::{Add, Sub};
    // Setup
    //let juno_to_user_after_discount = amt_before_discount.checked_multiply_ratio(100.sub(discount), 100_u32).map_err(|_| ContractError::GenericError("Calculate discount".to_string()))?;

    let discount = 10;

    // let amt_before_discount = Uint128::from(200_000_000_u32);

    let infofunds: Vec<Coin> = coins(200_000_000, "usdcx");

    ensure!(
        (infofunds[0].amount % Uint128::from(1000_u32) == Uint128::from(0_u32)),
        here(format!("xx: {}", "zzz".to_string()), line!(), column!())
    );

    // if infofunds[0].amount % Uint128::from(1000_u32) != Uint128::from(0_u32) {
    //     return Err(ContractError::GenericError("Amount sent not rounded to 3rd decimal".to_string()));
    // };

    // Determine the amount purchased, will be a query to oracle
    // Fake price for now: 10 USDC = 1 JUNO
    let fake_price = Uint128::from(10_000_000_u32);

    // 10_001_000
    let rounded_price = third_dec_ceil(fake_price)?;

    // 200_000_000
    let floored_funds = third_dec_floor(infofunds[0].amount)?;

    // 19 | actual 200_000_000 / 10_001_000 = 19.9980002
    let amt_before_discount = floored_funds.checked_div(rounded_price).unwrap();

    // 19_998_000
    let xxx =
        Uint128::from(1_000_000_u32).checked_multiply_ratio(floored_funds, rounded_price).unwrap();

    // 20_997_900
    let yyy = xxx.checked_multiply_ratio(100_u32.add(discount), 100_u32).unwrap();

    ////
    // this is calculating what 95% of what the juno amount is, needs to be 105%
    //let juno_to_user_after_discount = amt_before_discount.checked_multiply_ratio(100_u32.add(discount), 100_u32).unwrap();

    //ensure!((juno_to_user_after_discount < Uint128::from(0_u32)), here(format!("xx: {}", amt_before_discount.to_string()), line!(), column!()));

    //ensure!((1 > 10), here(format!("infofunds[0].amount: {}", infofunds[0].amount.to_string()), line!(), column!()));

    //ensure!((1 > 10), here(format!("rounded_price: {}", rounded_price.to_string()), line!(), column!()));

    //ensure!((1 > 10), here(format!("floored_funds: {}", floored_funds.to_string()), line!(), column!()));

    //ensure!((1 > 10), here(format!("amt_before_discount: {}", amt_before_discount.to_string()), line!(), column!()));

    ensure!(
        (1 > 10),
        here(format!("amount after discount: {}", yyy.to_string()), line!(), column!())
    );

    Ok(())
}
