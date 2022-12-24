#![cfg(test)]
use core::fmt::Display;
use anyhow::ensure;
use std::ops::Add;

use cosmwasm_std::{coins, to_binary, Addr, Binary, Coin, Empty, Uint128, coin};

// use self::create_contract::*;
// use self::create_users::*;
// use self::init_contracts::init_all_contracts;
use crate::{msg::*, state::*, integration_tests::setup_users::fake_user};

use cw_multi_test::{App, Contract, ContractWrapper, Executor};
//use self::create_contract::*;


pub fn here(ctx: impl Display, line: impl Display, col: impl Display) -> String {
    format!(
        "~~~~~~~~~~~~~~~~~~~ \n \n {} \n line {} | column {} \n ________________________",
        ctx, line, col
    )
}


const NATIVE_JUNO: &str = "ujunox";
const NATIVE_USDC: &str = "usdcx";
const NATIVE_INVALID: &str = "fakex";

pub mod setup_users {
    //use crate::integration_tests::{Contract, ContractWrapper, Empty};
    use super::{NATIVE_USDC, NATIVE_INVALID};
    use cosmwasm_std::Addr;
    use cw_multi_test::App;
    use std::borrow::BorrowMut;


    pub struct User {
        pub name: String,
        pub address: Addr,
    }

    pub fn fake_user(name: String) -> User {
        User {
            name: name.clone(),
            address: Addr::unchecked(format!("{}", name)),
        }
    }

    pub fn give_natives_to_user<'a>(user: &User, router: &'a mut App) -> &'a mut App {
        let usdc = cosmwasm_std::coin(100_000_000, NATIVE_USDC);
        let invalid_native = cosmwasm_std::coin(100_000_000, NATIVE_INVALID);

        router.borrow_mut().init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &user.address, vec![usdc, invalid_native])
                .unwrap()
        });

        router
    }


    pub fn setup(
        router: &mut App,
    ) -> (&mut App, User, User, User, User) {

        let admin = fake_user("admin".to_string());
        let john = fake_user("john".to_string());
        let sam = fake_user("sam".to_string());
        let max = fake_user("max".to_string());

        let rx = give_natives_to_user(&john, router);
        let ry = give_natives_to_user(&sam, rx);
        let rz = give_natives_to_user(&max, ry);

        (rz, admin, john, sam, max)

    }
}

pub mod setup_contract {
    use super::*;
    use std::marker::PhantomData;
    use std::borrow::BorrowMut;
    use super::{NATIVE_JUNO, NATIVE_USDC};

    pub fn cpbond_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }


    pub fn init_cpbond(
        router: &mut App,
        admin: &Addr,
    ) -> Addr {
        let cpbond_id = router.store_code(cpbond_contract());
        let msg = InstantiateMsg {
            admin: None,
            beingsold_denom: NATIVE_JUNO.to_string(),
            cost_denom: NATIVE_USDC.to_string()
        };

        let addr =
            router.instantiate_contract(cpbond_id, admin.clone(), &msg, &[], "cp_bond", None).unwrap();

        println!("cp_bond | Addr: {:?}", addr);

        addr
    }


    pub fn give_juno_to_contract<'a>(cpbond_contract: &Addr, router: &'a mut App) -> &'a mut App {
        let juno = cosmwasm_std::coin(1_000_000_000, NATIVE_JUNO);

        router.borrow_mut().init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(storage, cpbond_contract, vec![juno])
                .unwrap()
        });

        router
    }


    pub fn setup<'a>(
        router: &'a mut App,
        admin: &Addr
    ) -> (&'a mut App, Addr) {

        let cpbond = init_cpbond(router, admin);

        let r = give_juno_to_contract(&cpbond, router);

        (r, cpbond)
    }


}

#[test]
fn test_setup() -> Result<(), anyhow::Error> {
    use std::borrow::BorrowMut;
    use crate::utils::*;
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    use std::ops::{Add, Sub};

    //~~~~~~~~~~~~~~~~~~~~
    // Setup
    //~~~~~~~~~~~~~~~~~~~~
    let mut router = App::default();
    // Users
    let (router, admin, john, sam, max) = setup_users::setup(&mut router);
    // Contract
    let (router, cpbond_contract) = setup_contract::setup(router, &admin.address);


    // Does contract have correct amount of JUNO
    let contract_balance: Coin = router.wrap().query_balance(cpbond_contract.to_string(), NATIVE_JUNO).unwrap();
    ensure!(
        (contract_balance.amount == Uint128::from(1_000_000_000_u128)),
        here(format!("contract balance: {} | denom: {}", contract_balance.amount, contract_balance.denom), line!(), column!())
    );

    // Do users have correct amount of USDC
    let john_balance: Coin = router.wrap().query_balance(john.address.to_string(), NATIVE_USDC).unwrap();
    ensure!(
        (john_balance.amount == Uint128::from(100_000_000_u128)),
        here(format!("john balance: {} | denom: {}", john_balance.amount, john_balance.denom), line!(), column!())
    );

    Ok(())
}


#[test]
fn test_purchase() -> Result<(), anyhow::Error> {
    use std::borrow::BorrowMut;
    use crate::utils::*;
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    use std::ops::{Add, Sub};

    //~~~~~~~~~~~~~~~~~~~~
    // Setup
    //~~~~~~~~~~~~~~~~~~~~
    let mut router = App::default();
    // Users
    let (router, admin, john, sam, max) = setup_users::setup(&mut router);
    // Contract
    let (router, cpbond_contract) = setup_contract::setup(router, &admin.address);


    // First test, have a user buy 10 USDCX with a 10 week vesting period

    // Make sure that the total they get is 1.1 JUNO

    // Increase the block height and test to make sure the claims work

    // 



    Ok(())

}



// Tests

// ~~~~~~~ Purchasing High Level ~~~~~~~~ //
// (X) - CHECK: purchase doesn't go through if vest period > MAX_VEST
// (X) - CHECK: purchase fails if denom send != cost_denom
// (X) - CHECK: purchase fails if coins sent == empty
// (X) - CHECK: purchase fails if >1 coin sent
// (X) - CHECK: purchase fails if amount > max_purchase_amount
// (X) - CHECK: purchase fails if purchase amount not rounded to 3rd decimal
#[test]
fn test_purchasing_all_failures() -> Result<(), anyhow::Error> {
    use std::borrow::BorrowMut;
    use crate::utils::*;
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    use std::ops::{Add, Sub};

    //~~~~~~~~~~~~~~~~~~~~
    // Setup
    //~~~~~~~~~~~~~~~~~~~~
    let mut router = App::default();
    // Users
    let (router, admin, john, sam, max) = setup_users::setup(&mut router);
    // Contract
    let (router, cpbond_contract) = setup_contract::setup(router, &admin.address);


    // contract has 1_000_000_000 JUNO (1,000 JUNO)
    // each user has 100_000_000 USDC (100 USDC)
    // fake price is 5_000_000 (5 USDC per JUNO)
    // max_purchase is 500_000_000 (500 USDC)

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // CHECK: purchase fails if amount > max_purchase_amount
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let whale = fake_user("whale".to_string());
    let alot_usdc = cosmwasm_std::coin(10_000_001_000, NATIVE_USDC);
    router.borrow_mut().init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &whale.address, vec![alot_usdc])
            .unwrap()
    });

    let buy_msg = crate::msg::ExecuteMsg::Purchase { vesting_period: 10_u128 };

    let res: Result<AppResponse> = router.execute_contract(
        whale.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(500_001_000, NATIVE_USDC),
    );
    ensure!(res.is_err(), here("Purchase amount > Max purchase failure", line!(), column!()));


    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // CHECK: Purchase fails if amount not rounded to 3rd decimal
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let buy_msg = crate::msg::ExecuteMsg::Purchase { vesting_period: 10_u128 };

    let res: Result<AppResponse> = router.execute_contract(
        whale.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(100_000_900, NATIVE_USDC),
    );
    ensure!(res.is_err(), here("Not rounded to 3rd decimal", line!(), column!()));


    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // CHECK: Purchase fails if >1 coin type sent
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let usd = coin(10, NATIVE_USDC);
    let fkx = coin(10, NATIVE_INVALID);

    let buy_msg = crate::msg::ExecuteMsg::Purchase { vesting_period: 10_u128 };

    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &[usd, fkx],
    );
    ensure!(res.is_err(), here("More than 1 coin sent", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // CHECK: Purchase fails if coins sent is empty
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let buy_msg = crate::msg::ExecuteMsg::Purchase { vesting_period: 10_u128 };

    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &[],
    );
    ensure!(res.is_err(), here("More than 1 coin sent", line!(), column!()));


    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // CHECK: Purchase fails if coins sent != cost_denom
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let buy_msg = crate::msg::ExecuteMsg::Purchase { vesting_period: 10_u128 };

    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(10, NATIVE_INVALID),
    );
    ensure!(res.is_err(), here("Wrong denom", line!(), column!()));

    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // CHECK: purchase doesn't go through if vest period > MAX_VEST
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    let buy_msg = crate::msg::ExecuteMsg::Purchase { vesting_period: 53_u128 };

    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(10, NATIVE_USDC),
    );
    ensure!(res.is_err(), here("Vest period > MAX_VEST", line!(), column!()));




    let contract_balance: Coin =
        router.wrap().query_balance(cpbond_contract.to_string(), "ujunox").unwrap();

    let res: Result<AppResponse> = router.execute_contract(
        whale.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(500_000_000, NATIVE_USDC),
    );
    // Juno to user:     10_998_900_000
    // contract_balance: 10_000_000_000
    ensure!(res.is_err(), here(format!("cbalance: {} || {:#?}", contract_balance.amount, res), line!(), column!()));



    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    // CHECK: purchase doesn't go through if purchase amount > amount contract has
    // 
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    Ok(())
}



// (X) - CHECK: user can't have more than 5 active purchases
#[test]
pub fn test_user_over_5_purchases() -> Result<(), anyhow::Error> {
    use std::borrow::BorrowMut;
    use crate::utils::*;
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    use std::ops::{Add, Sub};

    //~~~~~~~~~~~~~~~~~~~~
    // Setup
    //~~~~~~~~~~~~~~~~~~~~
    let mut router = App::default();
    // Users
    let (router, admin, john, sam, max) = setup_users::setup(&mut router);
    // Contract
    let (router, cpbond_contract) = setup_contract::setup(router, &admin.address);

    // contract has 1_000_000_000 JUNO (1,000 JUNO)
    // each user has 100_000_000 USDC (100 USDC)
    // fake price is 5_000_000 (5 USDC per JUNO)
    // max_purchase is 500_000_000 (500 USDC)

    let buy_msg = crate::msg::ExecuteMsg::Purchase { vesting_period: 10_u128 };

    // John makes 5 purchases
    for x in 0..=4 {
        let res: Result<AppResponse> = router.execute_contract(
            john.address.clone(),
            cpbond_contract.clone(),
            &buy_msg,
            &coins(1_000_000, NATIVE_USDC),
        );
        ensure!(res.is_ok(), here(format!("john buy {}", x), line!(), column!()));
    }

    // Sixth Purchase fails
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(1_000_000, NATIVE_USDC),
    );
    ensure!(res.is_err(), here("John's sixth buy should've failed", line!(), column!()));

    // Fast forward 10 weeks * 100_000 blocks per week (1_000_000 blocks)
    router.update_block(|current_blockinfo| {
        current_blockinfo.height += 1_000_000;
        current_blockinfo.time = current_blockinfo.time.plus_seconds(6_000_000);
    });


    // Vest 1 purchase
    let claim_msg = crate::msg::ExecuteMsg::ClaimAvailable { uuid: 1_u64 };
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &claim_msg,
        &[]
    );
    ensure!(res.is_ok(), here("Vest a purchase", line!(), column!()));

    let q = crate::msg::QueryMsg::GetUserPurchaseIds { user_wallet: john.address.to_string() };
    let qrez: crate::query::UserPurchaseIds = {
        let qres: Binary = router.wrap().query_wasm_smart(cpbond_contract.clone(), &q).unwrap();
        cosmwasm_std::from_binary(&qres).unwrap()
    };

    ensure!((qrez.uuids.len() == 4), here(format!("Johns active vests: {:#?}", qrez.uuids), line!(), column!()));

    // Should now be able to make another purchase
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(1_000_000, NATIVE_USDC),
    );
    ensure!(res.is_ok(), here("Buy after vest / remove a purchase should've worked", line!(), column!()));

    // But not 2 more
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(1_000_000, NATIVE_USDC),
    );
    ensure!(res.is_err(), here("Buy after vest / remove a purchase should've worked", line!(), column!()));

    // John vests another one
    let claim_msg = crate::msg::ExecuteMsg::ClaimAvailable { uuid: 2_u64 };
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &claim_msg,
        &[]
    );
    ensure!(res.is_ok(), here("Vest a purchase", line!(), column!()));

    // Now he should be fine
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(1_000_000, NATIVE_USDC),
    );
    ensure!(res.is_ok(), here("Buy after vest / remove a purchase should've worked", line!(), column!()));


    Ok(())

}

// ( ) - CHECK: purchase doesn't go through if purchase amount > amount contract has
#[test]
pub fn test_purchase_over_contract_balance() -> Result<(), anyhow::Error> {
    use std::borrow::BorrowMut;
    use crate::utils::*;
    use anyhow::Result;
    use cw_multi_test::AppResponse;
    use std::ops::{Add, Sub};

    //~~~~~~~~~~~~~~~~~~~~
    // Setup
    //~~~~~~~~~~~~~~~~~~~~
    let mut router = App::default();
    // Users
    let (router, admin, john, sam, max) = setup_users::setup(&mut router);
    // Contract
    let (router, cpbond_contract) = setup_contract::setup(router, &admin.address);

    // contract has 1_000_000_000 JUNO (1,000 JUNO)
    // each user has 100_000_000 USDC (100 USDC)
    // fake price is 5_000_000 (5 USDC per JUNO)
    // max_purchase is 500_000_000 (500 USDC)

    // Give whale 10_000_000_000 USDC (10,000 USDC)
    let whale = fake_user("whale".to_string());
    let alot_usdc = cosmwasm_std::coin(10_000_000_000, NATIVE_USDC);
    router.borrow_mut().init_modules(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &whale.address, vec![alot_usdc])
            .unwrap()
    });

    // Whale buys 

    let buy_msg = crate::msg::ExecuteMsg::Purchase { vesting_period: 10_u128 };

    // John makes 5 purchases
    for x in 0..=4 {
        let res: Result<AppResponse> = router.execute_contract(
            john.address.clone(),
            cpbond_contract.clone(),
            &buy_msg,
            &coins(1_000_000, NATIVE_USDC),
        );
        ensure!(res.is_ok(), here(format!("john buy {}", x), line!(), column!()));
    }

    // Sixth Purchase fails
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &buy_msg,
        &coins(1_000_000, NATIVE_USDC),
    );
    ensure!(res.is_err(), here("John's sixth buy should've failed", line!(), column!()));

    // Fast forward 10 weeks * 100_000 blocks per week (1_000_000 blocks)
    router.update_block(|current_blockinfo| {
        current_blockinfo.height += 1_000_000;
        current_blockinfo.time = current_blockinfo.time.plus_seconds(6_000_000);
    });


    // Vest 1 purchase
    let claim_msg = crate::msg::ExecuteMsg::ClaimAvailable { uuid: 1_u64 };
    let res: Result<AppResponse> = router.execute_contract(
        john.address.clone(),
        cpbond_contract.clone(),
        &claim_msg,
        &[]
    );
    ensure!(res.is_ok(), here("Vest a purchase", line!(), column!()));

    let q = crate::msg::QueryMsg::GetUserPurchaseIds { user_wallet: john.address.to_string() };
    let qrez: crate::query::UserPurchaseIds = {
        let qres: Binary = router.wrap().query_wasm_smart(cpbond_contract.clone(), &q).unwrap();
        cosmwasm_std::from_binary(&qres).unwrap()
    };

    ensure!((qrez.uuids.len() == 4), here(format!("Johns active vests: {:#?}", qrez.uuids), line!(), column!()));


    Ok(())
}


// Into the trenches of edge cases we go...

// Check that vesting claims give correct amount

