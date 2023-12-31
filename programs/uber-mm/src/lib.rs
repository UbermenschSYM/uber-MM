use anchor_lang::prelude::*;
use oracle::*;
use consts::*;
pub mod oracle;
pub mod consts;
use anchor_lang::{
    __private::bytemuck::{self},
    solana_program::program::{get_return_data, invoke},
};
use phoenix::program::{
    new_order::{CondensedOrder, MultipleOrderPacket},
    CancelMultipleOrdersByIdParams, CancelOrderParams, MarketHeader,
};
use phoenix::{
    quantities::WrapperU64,
    state::{
        markets::{FIFOOrderId, FIFORestingOrder, Market},
        OrderPacket, Side,
    },
};

declare_id!("Exz7z8HpBjS7trD6ZbdWABdQyhK5ZvGkuV4UYoUiSTQQ");

#[derive(Clone)]
pub struct PhoenixV1;

impl anchor_lang::Id for PhoenixV1 {
    fn id() -> Pubkey {
        phoenix::id()
    }
}


#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
struct DeserializedFIFOOrderId {
    pub price_in_ticks: u64,
    pub order_sequence_number: u64,
}

fn parse_order_ids_from_return_data(order_ids: &mut Vec<FIFOOrderId>) -> Result<()> {
    if let Some((program_id, orders_data)) = get_return_data() {
        msg!("Found return data");
        if program_id == phoenix::id() && !orders_data.is_empty() {
            msg!("Found orders in return data");
            Vec::<DeserializedFIFOOrderId>::try_from_slice(&orders_data)?
                .into_iter()
                .for_each(|o| {
                    order_ids.push(FIFOOrderId::new_from_untyped(
                        o.price_in_ticks,
                        o.order_sequence_number,
                    ))
                });
        } else {
            msg!("No orders in return data");
        }
    }
    Ok(())
}

fn load_header(info: &AccountInfo) -> Result<MarketHeader> {
    require!(
        info.owner == &phoenix::id(),
        StrategyError::InvalidPhoenixProgram
    );
    let data = info.data.borrow();
    let header =
        bytemuck::try_from_bytes::<MarketHeader>(&data[..std::mem::size_of::<MarketHeader>()])
            .map_err(|_| {
                msg!("Failed to parse Phoenix market header");
                StrategyError::FailedToDeserializePhoenixMarket
            })?;
    require!(
        header.discriminant == PHOENIX_MARKET_DISCRIMINANT,
        StrategyError::InvalidPhoenixProgram,
    );
    Ok(*header)
}

fn get_best_bid_and_ask(
    market: &dyn Market<Pubkey, FIFOOrderId, FIFORestingOrder, OrderPacket>,
    trader_index: u64,
) -> (u64, u64) {
    let best_bid = market
        .get_book(Side::Bid)
        .iter()
        .find(|(_, o)| o.trader_index != trader_index)
        .map(|(o, _)| o.price_in_ticks.as_u64())
        .unwrap_or_else(|| 1);
    let best_ask = market
        .get_book(Side::Ask)
        .iter()
        .find(|(_, o)| o.trader_index != trader_index)
        .map(|(o, _)| o.price_in_ticks.as_u64())
        .unwrap_or_else(|| u64::MAX);
    (best_bid, best_ask)
}

fn get_bid_price_in_ticks(
    fair_price_in_ticks: u64,
    header: &MarketHeader,
    edge_in_bps: u64,
) -> u64 {
    let edge_in_ticks = edge_in_bps * fair_price_in_ticks / 10_000;
    fair_price_in_ticks - edge_in_ticks
}

fn get_ask_price_in_ticks(
    fair_price_in_ticks: u64,
    header: &MarketHeader,
    edge_in_bps: u64,
) -> u64 {
    let edge_in_ticks = edge_in_bps * fair_price_in_ticks / 10_000;
    fair_price_in_ticks + edge_in_ticks
}

fn get_fair_price_in_ticks(
    base_price: u128,
    quote_price: u128,
    header: &MarketHeader,
) -> u64 {
    let fair_price_in_ticks = (base_price * (u64::pow(10, header.quote_params.decimals as u32) as u128)
        * header.raw_base_units_per_base_unit as u128
        / header.get_tick_size_in_quote_atoms_per_base_unit().as_u128() / quote_price) as u64;
    fair_price_in_ticks
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum PriceImprovementBehavior {
    Ubermensch,
    Join,
    Dime,
    Ignore,
}

impl PriceImprovementBehavior {
    pub fn to_u8(&self) -> u8 {
        match self {
            PriceImprovementBehavior::Ubermensch => 0,
            PriceImprovementBehavior::Join => 1,
            PriceImprovementBehavior::Dime => 2,
            PriceImprovementBehavior::Ignore => 3,
        }
    }

    pub fn from_u8(byte: u8) -> Self {
        match byte {
            0 => PriceImprovementBehavior::Ubermensch,
            1 => PriceImprovementBehavior::Join,
            2 => PriceImprovementBehavior::Dime,
            3 => PriceImprovementBehavior::Ignore,
            _ => panic!("Invalid PriceImprovementBehavior"),
        }
    }
}

#[account(zero_copy)]
pub struct PhoenixStrategyState {
    pub trader: Pubkey,
    pub market: Pubkey,
    // Order parameters
    pub bid_order_sequence_number: u64,
    pub bid_price_in_ticks: u64,
    pub initial_bid_size_in_base_lots: u64,
    pub ask_order_sequence_number: u64,
    pub ask_price_in_ticks: u64,
    pub initial_ask_size_in_base_lots: u64,
    pub last_update_slot: u64,
    pub last_update_unix_timestamp: i64,
    // Strategy parameters
    /// Number of basis points betweeen quoted price and fair price
    pub quote_edge_in_bps: u64,
    /// Order notional size in quote atoms
    pub quote_size_in_quote_atoms: u64,
    /// If set to true, the orders will never cross the spread
    pub post_only: bool,
    /// Determines whether/how to improve BBO
    pub price_improvement_behavior: u8,
    padding: [u8; 6],
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct OrderParams {
    pub fair_price_in_quote_atoms_per_raw_base_unit: u64,
    pub strategy_params: StrategyParams,
    pub use_oracle: bool,
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct StrategyParams {
    pub quote_edge_in_bps: Option<u64>,
    pub quote_size_in_quote_atoms: Option<u64>,
    pub price_improvement_behavior: Option<PriceImprovementBehavior>,
    pub post_only: Option<bool>,
}

#[program]
pub mod uber_mm {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>, 
        quote_edge_in_bps: u64, 
        quote_size_in_quote_atoms: u64,
        price_improvement_behavior: u8,
        post_only: bool,
    ) -> Result<()> {
        require!(
            quote_edge_in_bps > 0,
            StrategyError::EdgeMustBeNonZero
        );
        load_header(&ctx.accounts.market)?;
        let clock = Clock::get()?;
        msg!("Initializing Phoenix Strategy");
        let mut phoenix_strategy = ctx.accounts.phoenix_strategy.load_init()?;
        *phoenix_strategy = PhoenixStrategyState {
            trader: *ctx.accounts.user.key,
            market: *ctx.accounts.market.key,
            bid_order_sequence_number: 0,
            bid_price_in_ticks: 0,
            initial_bid_size_in_base_lots: 0,
            ask_order_sequence_number: 0,
            ask_price_in_ticks: 0,
            initial_ask_size_in_base_lots: 0,
            last_update_slot: clock.slot,
            last_update_unix_timestamp: clock.unix_timestamp,
            quote_edge_in_bps: quote_edge_in_bps,
            quote_size_in_quote_atoms: quote_size_in_quote_atoms,
            post_only: post_only,
            price_improvement_behavior: price_improvement_behavior,
            padding: [0; 6],
        };
        Ok(())
    }

    pub fn update_quotes(
        ctx: Context<UpdateQuotes>,
        fair_price_in_quote_atoms_per_raw_base_unit: u64,
        quote_edge_in_bps: u64, 
        quote_size_in_quote_atoms: u64,
        price_improvement_behavior: u8,
        post_only: bool,
        use_oracle: bool,
        margin: u64,
    ) -> Result<()> {
        let UpdateQuotes {
            phoenix_strategy,
            user,
            phoenix_program,
            log_authority,
            market: market_account,
            seat,
            quote_account,
            base_account,
            quote_vault,
            base_vault,
            token_program,
        } = ctx.accounts;

        let mut phoenix_strategy = phoenix_strategy.load_mut()?;

        // Update timestamps
        let clock = Clock::get()?;
        phoenix_strategy.last_update_slot = clock.slot;
        phoenix_strategy.last_update_unix_timestamp = clock.unix_timestamp;

        // Update the strategy parameters
        if let edge = quote_edge_in_bps {
            if edge > 0 {
                phoenix_strategy.quote_edge_in_bps = edge;
            }
        }
        if let size = quote_size_in_quote_atoms {
            phoenix_strategy.quote_size_in_quote_atoms = size;
        }
        if let post_only = post_only {
            phoenix_strategy.post_only = post_only;
        }
        if let price_improvement_behavior = price_improvement_behavior
        {
            phoenix_strategy.price_improvement_behavior = price_improvement_behavior;
        }

        // Load market
        let header = load_header(market_account)?;
        let market_data = market_account.data.borrow();
        let (_, market_bytes) = market_data.split_at(std::mem::size_of::<MarketHeader>());
        let market = phoenix::program::load_with_dispatch(&header.market_size_params, market_bytes)
            .map_err(|_| {
                msg!("Failed to deserialize market");
                StrategyError::FailedToDeserializePhoenixMarket
            })?
            .inner;
        let mut fair_price_in_ticks = fair_price_in_quote_atoms_per_raw_base_unit;

        // checking if oracle is used to calculate the fair price

        if use_oracle {
            msg!("Using oracle to calculate the fair price");
            let mut pyth_account: &AccountInfo = &ctx.remaining_accounts[0];
            let mut oracle_price = Price::load(&pyth_account)?;
            msg!("oracle price = {}, oracle expo = {}", oracle_price.price, oracle_price.expo);
            // calculating the price by multiplying oracle price on 10^6 and dividing it on 10^expo
            let base_fair_price = (BIG_NUMBER as u128 * oracle_price.price as u128 / (u64::pow(10, (-oracle_price.expo) as u32) as u128)) as u128;

            pyth_account = &ctx.remaining_accounts[1];
            oracle_price = Price::load(&pyth_account)?;

            let quote_fair_price = (BIG_NUMBER as u128 * oracle_price.price as u128 / (u64::pow(10, (-oracle_price.expo) as u32) as u128)) as u128;
            msg!("Base price = {}, quote price = {}", base_fair_price, quote_fair_price);

            fair_price_in_ticks = get_fair_price_in_ticks(
                base_fair_price,
                quote_fair_price,
                &header,
            );
        }


        msg!("{} {}", header.raw_base_units_per_base_unit as u64, header.get_tick_size_in_quote_atoms_per_base_unit().as_u64());

        // Compute quote prices
        let mut bid_price_in_ticks = get_bid_price_in_ticks(
            fair_price_in_ticks,
            &header,
            phoenix_strategy.quote_edge_in_bps,
        );

        let mut ask_price_in_ticks = get_ask_price_in_ticks(
            fair_price_in_ticks,
            &header,
            phoenix_strategy.quote_edge_in_bps,
        );

        // Returns the best bid and ask prices that are not placed by the trader
        let trader_index = market.get_trader_index(&user.key()).unwrap_or(u32::MAX) as u64;
        let (best_bid, best_ask) = get_best_bid_and_ask(market, trader_index);

        msg!("Current market: {} @ {}, our: {} {}", best_bid, best_ask, bid_price_in_ticks, ask_price_in_ticks);
        msg!("fair price: {}", fair_price_in_ticks);
        let price_improvement_behavior =
            PriceImprovementBehavior::from_u8(phoenix_strategy.price_improvement_behavior);
        match price_improvement_behavior {
            PriceImprovementBehavior::Ubermensch => {
                // we check current ask/bid price's relationship to fair price
                // if difference between ask/bid price and fair price is more than margin
                // then we still trade
                ask_price_in_ticks = ask_price_in_ticks.max(best_ask);
                bid_price_in_ticks = bid_price_in_ticks.min(best_bid);
                if best_ask > fair_price_in_ticks + margin {
                    ask_price_in_ticks = best_ask;
                }

                if best_bid < fair_price_in_ticks - margin {
                    bid_price_in_ticks = best_bid;
                }

                msg!("ask price = {}, bid price = {}", ask_price_in_ticks, bid_price_in_ticks);

            }
            PriceImprovementBehavior::Join => {
                // If price_improvement_behavior is set to Join, we will always join the best bid and ask
                // if our quote prices are within the spread
                ask_price_in_ticks = ask_price_in_ticks.max(best_ask);
                bid_price_in_ticks = bid_price_in_ticks.min(best_bid);
            }
            PriceImprovementBehavior::Dime => {
                // If price_improvement_behavior is set to Dime, we will never price improve by more than 1 tick
                ask_price_in_ticks = ask_price_in_ticks.max(best_ask - 1);
                bid_price_in_ticks = bid_price_in_ticks.min(best_bid + 1);
            }
            PriceImprovementBehavior::Ignore => {
                // If price_improvement_behavior is set to Ignore, we will not update our quotes based off the current
                // market prices
            }
        }

        // Compute quote amounts in base lots
        let size_in_quote_lots =
            phoenix_strategy.quote_size_in_quote_atoms / header.get_quote_lot_size().as_u64();

        let bid_size_in_base_lots = size_in_quote_lots
            * market.get_base_lots_per_base_unit().as_u64()
            / (bid_price_in_ticks * market.get_tick_size().as_u64());
        let ask_size_in_base_lots = size_in_quote_lots
            * market.get_base_lots_per_base_unit().as_u64()
            / (ask_price_in_ticks * market.get_tick_size().as_u64());

        msg!(
            "Our market: {} {} @ {} {}",
            bid_size_in_base_lots,
            bid_price_in_ticks,
            ask_price_in_ticks,
            ask_size_in_base_lots
        );

        let mut update_bid = true;
        let mut update_ask = true;
        let orders_to_cancel = [
            (
                Side::Bid,
                bid_price_in_ticks,
                FIFOOrderId::new_from_untyped(
                    phoenix_strategy.bid_price_in_ticks,
                    phoenix_strategy.bid_order_sequence_number,
                ),
                phoenix_strategy.initial_bid_size_in_base_lots,
            ),
            (
                Side::Ask,
                ask_price_in_ticks,
                FIFOOrderId::new_from_untyped(
                    phoenix_strategy.ask_price_in_ticks,
                    phoenix_strategy.ask_order_sequence_number,
                ),
                phoenix_strategy.initial_ask_size_in_base_lots,
            ),
        ]
        .iter()
        .filter_map(|(side, price, order_id, initial_size)| {
            if let Some(resting_order) = market.get_book(*side).get(order_id) {
                // The order is 100% identical, do not cancel it
                if resting_order.num_base_lots == *initial_size
                    && order_id.price_in_ticks.as_u64() == *price
                {
                    msg!("Resting order is identical: {:?}", order_id);
                    match side {
                        Side::Bid => update_bid = false,
                        Side::Ask => update_ask = false,
                    }
                    return None;
                }
                msg!("Found partially filled resting order: {:?}", order_id);
                // The order has been partially filled or reduced
                return Some(*order_id);
            }
            msg!("Failed to find resting order: {:?}", order_id);
            // The order has been fully filled
            None
        })
        .collect::<Vec<FIFOOrderId>>();

        // Drop reference prior to invoking
        drop(market_data);

        // Cancel the old orders
        if !orders_to_cancel.is_empty() {
            invoke(
                &phoenix::program::create_cancel_multiple_orders_by_id_with_free_funds_instruction(
                    &market_account.key(),
                    &user.key(),
                    &CancelMultipleOrdersByIdParams {
                        orders: orders_to_cancel
                            .iter()
                            .map(|o_id| CancelOrderParams {
                                order_sequence_number: o_id.order_sequence_number,
                                price_in_ticks: o_id.price_in_ticks.as_u64(),
                                side: Side::from_order_sequence_number(o_id.order_sequence_number),
                            })
                            .collect::<Vec<_>>(),
                    },
                ),
                &[
                    phoenix_program.to_account_info(),
                    log_authority.to_account_info(),
                    user.to_account_info(),
                    market_account.to_account_info(),
                ],
            )?;
        }

        // Don't update quotes if the price is invalid or if the sizes are 0
        update_bid &= bid_price_in_ticks > 1 && bid_size_in_base_lots > 0;
        update_ask &= ask_price_in_ticks < u64::MAX && ask_size_in_base_lots > 0;

        let client_order_id = u128::from_le_bytes(user.key().to_bytes()[..16].try_into().unwrap());
        if !update_ask && !update_bid && orders_to_cancel.is_empty() {
            msg!("No orders to update");
            return Ok(());
        }
        let mut order_ids = vec![];
        if phoenix_strategy.post_only
            || !matches!(price_improvement_behavior, PriceImprovementBehavior::Join)
        {
            // Send multiple post-only orders in a single instruction
            let multiple_order_packet = MultipleOrderPacket::new(
                if update_bid {
                    vec![CondensedOrder::new_default(
                        bid_price_in_ticks,
                        bid_size_in_base_lots,
                    )]
                } else {
                    vec![]
                },
                if update_ask {
                    vec![CondensedOrder::new_default(
                        ask_price_in_ticks,
                        ask_size_in_base_lots,
                    )]
                } else {
                    vec![]
                },
                Some(client_order_id),
                false,
            );
            invoke(
                &phoenix::program::create_new_multiple_order_instruction_with_custom_token_accounts(
                    &market_account.key(),
                    &user.key(),
                    &base_account.key(),
                    &quote_account.key(),
                    &header.base_params.mint_key,
                    &header.quote_params.mint_key,
                    &multiple_order_packet,
                ),
                &[
                    phoenix_program.to_account_info(),
                    log_authority.to_account_info(),
                    user.to_account_info(),
                    market_account.to_account_info(),
                    seat.to_account_info(),
                    quote_account.to_account_info(),
                    base_account.to_account_info(),
                    quote_vault.to_account_info(),
                    base_vault.to_account_info(),
                    token_program.to_account_info(),
                ],
            )?;
            parse_order_ids_from_return_data(&mut order_ids)?;
        } else {
            if update_bid {
                invoke(
                    &phoenix::program::create_new_order_instruction_with_custom_token_accounts(
                        &market_account.key(),
                        &user.key(),
                        &base_account.key(),
                        &quote_account.key(),
                        &header.base_params.mint_key,
                        &header.quote_params.mint_key,
                        &OrderPacket::new_limit_order_default_with_client_order_id(
                            Side::Bid,
                            bid_price_in_ticks,
                            bid_size_in_base_lots,
                            client_order_id,
                        ),
                    ),
                    &[
                        phoenix_program.to_account_info(),
                        log_authority.to_account_info(),
                        user.to_account_info(),
                        market_account.to_account_info(),
                        seat.to_account_info(),
                        quote_account.to_account_info(),
                        base_account.to_account_info(),
                        quote_vault.to_account_info(),
                        base_vault.to_account_info(),
                        token_program.to_account_info(),
                    ],
                )?;
                parse_order_ids_from_return_data(&mut order_ids)?;
            }
            if update_ask {
                invoke(
                    &phoenix::program::create_new_order_instruction_with_custom_token_accounts(
                        &market_account.key(),
                        &user.key(),
                        &base_account.key(),
                        &quote_account.key(),
                        &header.base_params.mint_key,
                        &header.quote_params.mint_key,
                        &OrderPacket::new_limit_order_default_with_client_order_id(
                            Side::Ask,
                            ask_price_in_ticks,
                            ask_size_in_base_lots,
                            client_order_id,
                        ),
                    ),
                    &[
                        phoenix_program.to_account_info(),
                        log_authority.to_account_info(),
                        user.to_account_info(),
                        market_account.to_account_info(),
                        seat.to_account_info(),
                        quote_account.to_account_info(),
                        base_account.to_account_info(),
                        quote_vault.to_account_info(),
                        base_vault.to_account_info(),
                        token_program.to_account_info(),
                    ],
                )?;
                parse_order_ids_from_return_data(&mut order_ids)?;
            }
        }

        let market_data = market_account.data.borrow();
        let (_, market_bytes) = market_data.split_at(std::mem::size_of::<MarketHeader>());
        let market = phoenix::program::load_with_dispatch(&header.market_size_params, market_bytes)
            .map_err(|_| {
                msg!("Failed to deserialize market");
                StrategyError::FailedToDeserializePhoenixMarket
            })?
            .inner;

        for order_id in order_ids.iter() {
            let side = Side::from_order_sequence_number(order_id.order_sequence_number);
            match side {
                Side::Ask => {
                    market
                        .get_book(Side::Ask)
                        .get(&order_id)
                        .map(|order| {
                            msg!("Placed Ask Order: {:?}", order_id);
                            phoenix_strategy.ask_price_in_ticks = order_id.price_in_ticks.as_u64();
                            phoenix_strategy.ask_order_sequence_number =
                                order_id.order_sequence_number;
                            phoenix_strategy.initial_ask_size_in_base_lots =
                                order.num_base_lots.as_u64();
                        })
                        .unwrap_or_else(|| {
                            msg!("Ask order not found");
                        });
                }
                Side::Bid => {
                    market
                        .get_book(Side::Bid)
                        .get(&order_id)
                        .map(|order| {
                            msg!("Placed Bid Order: {:?}", order_id);
                            phoenix_strategy.bid_price_in_ticks = order_id.price_in_ticks.as_u64();
                            phoenix_strategy.bid_order_sequence_number =
                                order_id.order_sequence_number;
                            phoenix_strategy.initial_bid_size_in_base_lots =
                                order.num_base_lots.as_u64();
                        })
                        .unwrap_or_else(|| {
                            msg!("Bid order not found");
                        });
                }
            }
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds=[b"phoenix".as_ref(), user.key.as_ref(), market.key.as_ref()],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<PhoenixStrategyState>(),
    )]
    pub phoenix_strategy: AccountLoader<'info, PhoenixStrategyState>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: Checked in instruction
    pub market: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateQuotes<'info> {
    #[account(
        mut,
        seeds=[b"phoenix".as_ref(), user.key.as_ref(), market.key.as_ref()],
        bump,
    )]
    pub phoenix_strategy: AccountLoader<'info, PhoenixStrategyState>,
    pub user: Signer<'info>,
    pub phoenix_program: Program<'info, PhoenixV1>,
    /// CHECK: Checked in CPI
    pub log_authority: UncheckedAccount<'info>,
    /// CHECK: Checked in instruction and CPI
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    pub seat: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    #[account(mut)]
    pub quote_account: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    #[account(mut)]
    pub base_account: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    #[account(mut)]
    pub quote_vault: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    #[account(mut)]
    pub base_vault: UncheckedAccount<'info>,
    /// CHECK: Checked in CPI
    pub token_program: UncheckedAccount<'info>,
}

