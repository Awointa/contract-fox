#![no_std]

use soroban_sdk::contracttype;

pub mod types;

pub use types::{Campaign, CampaignStatus, Donation, Withdrawal};
