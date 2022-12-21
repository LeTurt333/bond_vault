use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw20::{Balance, Cw20CoinVerified};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex, UniqueIndex};

use cosmwasm_schema::cw_serde;

use crate::ContractError;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Config
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub const CONFIG: Item<Config> = Item::new("cp_b_config");

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub beingsold_denom: String,
    pub cost_denom: String,
}

// Running count of purchases, used for UUID's, reset at u64::MAX
pub const PURCHASE_COUNT: Item<u64> = Item::new("purchase_count");

// USERS stores (Addr, Purchase UUID)
// this way a user can purchase more than once,
// but each purchase will have it's own calculated expiration / amount / already_claimed
// --
// Then, when a user wants to claim their vestable amounts,
// That function finds every entry into the Map that has their address,
// and does the claim_vestable on each one


// primary key is (User Address, UUID of this purchase)
// Index Could be (User Address, Purchase #)

// Or
// Store a seperate Map that stores the number of purchases per Address & read from that
// Map<Addr, u64>

pub const PURCHASES: Map<(Addr, u64), Purchase> = Map::new("purchases");

#[cw_serde]
pub struct Purchase {
    pub vest_period: u8,           // in weeks, 1 = 1 week, 7 = 7 weeks...
    pub vest_expiration: u64,      // block height when purchase is fully vested
    pub amount_purchased: Uint128, // amount of JUNO originally purchased
    pub already_claimed: Uint128,  // amount of JUNO already claimed
    pub last_claim: u64, // block height of last claim, initially set to block_height when purchased
    //pub closed: bool,    // true if this purchase has been totally claimed
}


// pub struct PurchaseIndexes<'a> {
//     pub uuid: UniqueIndex<'a, String, Purchase, (&'a Addr, String)>,
//     pub finalized_date: MultiIndex<'a, u64, Purchase, (&'a Addr, String)>,

//     // key will be (User Address, UUID)
//     // Unique Index of uuid
//     // MultiIndex of user number of purchases
//     // So I can do Prefix XXX

    
//     // How about an Index Map where the Key is just User Address,
//     // then 2 Indexes
//     // UniqueIndex - UUID of each purchase
//     // MultiIndex - User number of purchases

//     // Then I can go like
//     // MAP.prefix(user_address).idx.number_of_purchases.range(none none).collect().count()


//     // Actually no, just do
//     // Map with (Addr, UUID), Unique Index of UUID


// }

// impl IndexList<Purchase> for PurchaseIndexes<'_> {
//     fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Purchase>> + '_> {
//         let v: Vec<&dyn Index<Purchase>> = vec![&self.id, &self.finalized_date];
//         Box::new(v.into_iter())
//     }
// }

// pub fn purchases<'a>() -> IndexedMap<'a, (&'a Addr, String), Purchase, PurchaseIndexes<'a>> {
//     let indexes = ListingIndexes {

//         id: UniqueIndex::new(|a_listing| a_listing.id.clone(), "listing__id"),

//         finalized_date: MultiIndex::new(
//             |_pk, a_listing| match a_listing.finalized_time {
//                 None => 0_u64,
//                 Some(x) => x.seconds() as u64,
//             },
//             "listings_im",
//             "listing__finalized__date",
//         ),
//     };

//     IndexedMap::new("listings_im", indexes)
// }














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

//

// SALES
// Each "Sale" period, each wallet is only allowed 1 purchase,
// with a maximum value of CONFIG.max_buy

// In the Sales Map we track each Sale & User Wallet to ensure they have only made 1 purchase,
// Maximum value is checked in the contract execution
