use crate::*;
use anchor_lang::prelude::AccountInfo;
use bytemuck::{cast_slice_mut, from_bytes_mut, try_cast_slice_mut, Pod, Zeroable};
use std::cell::{RefMut};

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct AccKey {
    pub val: [u8; 32],
}

#[derive(PartialEq, Debug, Copy, Clone)]
#[repr(C)]
pub enum PriceStatus {
    Unknown = 0,
    Trading = 1,
    Halted = 2,
    Auction = 3,
}

impl Default for PriceStatus {
    fn default() -> Self {
        PriceStatus::Trading
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum CorpAction {
    NoCorpAct,
}

impl Default for CorpAction {
    fn default() -> Self {
        CorpAction::NoCorpAct
    }
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct PriceInfo {
    pub price: i64,
    pub conf: u64,
    pub status: PriceStatus,
    pub corp_act: CorpAction,
    pub pub_slot: u64,
}
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct PriceComp {
    publisher: AccKey,
    agg: PriceInfo,
    latest: PriceInfo,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum PriceType {
    Unknown,
    Price,
    TWAP,
    Volatility,
}

impl Into<u8> for PriceStatus {
    fn into(self) -> u8 {
        match self {
            PriceStatus::Unknown => 0,
            PriceStatus::Trading => 1,
            PriceStatus::Halted => 2,
            PriceStatus::Auction => 3,
        }
    }
}

impl Default for PriceType {
    fn default() -> Self {
        PriceType::Price
    }
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Ema {
    pub val: i64, // Current value of ema
    numer: i64,   // Numerator state for next update
    denom: i64,   // Denominator state for next update
}
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Price {
    pub magic: u32,            // Pyth magic number
    pub ver: u32,              // Program version
    pub atype: u32,            // Account type
    pub size: u32,             // Price account size
    pub ptype: PriceType,      // Price or calculation type
    pub expo: i32,             // Price exponent
    pub num: u32,              // Number of component prices
    pub num_qt: u32,           // Number of quoters that make up aggregate
    pub last_slot: u64,        // Slot of last valid (not unknown) aggregate price
    pub valid_slot: u64,       // Valid slot-time of agg. price
    pub twap: Ema,             // Time-weighted average price
    pub twac: Ema,             // Time-weighted average confidence interval
    pub drv1: i64,             // Space for future derived values
    pub drv2: i64,             // Space for future derived values
    pub prod: AccKey,          // Product account key
    pub next: AccKey,          // Next Price account in linked list
    pub prev_slot: u64,        // Valid slot of previous update
    pub prev_price: i64,       // Aggregate price of previous update
    pub prev_conf: u64,        // Confidence interval of previous update
    pub drv3: i64,             // Space for future derived values
    pub agg: PriceInfo,        // Aggregate price info
    pub comp: [PriceComp; 32], // Price components one per quoter
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct SimplePriceInfo {
    pub price: i64,
    pub low: i64,
    pub high: i64,
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct SimplePrice {
    pub expo: i32,                      // Price exponent
    pub price: i64,                     // Aggregate price info
}

impl Price {
    #[inline]
    pub fn load<'a>(price_feed: &'a AccountInfo) -> Result<SimplePrice> {
        let account_data: RefMut<'a, [u8]>;
        let state: RefMut<'a, Self>;

        account_data = RefMut::map(price_feed.try_borrow_mut_data().unwrap(), |data| *data);

        state = RefMut::map(account_data, |data| {
            from_bytes_mut(cast_slice_mut::<u8, u8>(try_cast_slice_mut(data).unwrap()))
        });

        if Clock::get()?.slot - state.valid_slot >= 50 {
            return Err(error!(StrategyError::PythValidSlot))
        }
        if state.agg.status != PriceStatus::Trading {
            return Err(error!(StrategyError::PythStatus))
        }
        if state.agg.price < 0 {
            return Err(error!(StrategyError::PythNegativePrice))
        }
        if state.agg.conf.checked_mul(10).unwrap() > state.agg.price as u64 {
            return Err(error!(StrategyError::PythConfidence));
        }
        let simple_price = SimplePrice {
            expo: state.expo,
            price: state.agg.price,
        };
        Ok(simple_price)
    }
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for Price {}

#[cfg(target_endian = "little")]
unsafe impl Pod for Price {}
