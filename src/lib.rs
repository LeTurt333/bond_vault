pub mod contract;
pub mod error;
use cosmwasm_std::Uint128;

pub use crate::error::ContractError;
pub mod integration_tests;
pub mod msg;
pub mod state;
pub mod utils;
pub mod query;

pub const FAKE_PRICE: Uint128 = Uint128::new(5_000_000_u128);

// Any arbitrary value, can be updatable by putting in Config instead of const
pub const MAX_PURCHASE_AMOUNT: u128 = 500_000_000_u128;

pub const MAX_VEST: u128 = 52_u128;
pub const BLOCKS_IN_WEEK: u128 = 100_000_u128;
