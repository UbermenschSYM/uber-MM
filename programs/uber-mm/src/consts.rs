use anchor_lang::prelude::*;
pub const PHOENIX_MARKET_DISCRIMINANT: u64 = 8167313896524341111;

pub const BIG_NUMBER: u128 = 1000000000000;

#[error_code]
pub enum StrategyError {
    NoReturnData,
    InvalidStrategyParams,
    EdgeMustBeNonZero,
    InvalidPhoenixProgram,
    FailedToDeserializePhoenixMarket,
    PythStatus,
    PythValidSlot,
    PythNegativePrice,
    PythConfidence,
}