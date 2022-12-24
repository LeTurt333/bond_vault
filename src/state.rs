use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use cosmwasm_schema::cw_serde;

pub const CONFIG: Item<Config> = Item::new("cp_b_config");

// If true, ExecuteMsg::Purchase cannot be called
pub const PAUSED: Item<bool> = Item::new("paused");

// Balance of contract in beingsold_denom
pub const BALANCE: Item<Uint128> = Item::new("balance");

// Running count of all purchases, used for UUID's, reset at u64::MAX
pub const PURCHASE_COUNT: Item<u64> = Item::new("purchase_count");

// Map of all purchases still active
pub const PURCHASES: Map<(Addr, u64), Purchase> = Map::new("purchases");


#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub beingsold_denom: String,
    pub cost_denom: String,
}

#[cw_serde]
pub struct Purchase {
    pub vest_period: u8,           // in weeks, 1 = 1 week, 7 = 7 weeks...
    pub vest_expiration: u64,      // block height when purchase is fully vested
    pub amount_purchased: Uint128, // amount of JUNO originally purchased
    pub already_claimed: Uint128,  // amount of JUNO already claimed
    pub last_claim: u64, // block height of last claim, initially set to block_height when purchased
    //pub closed: bool,    // true if this purchase has been totally claimed
}


