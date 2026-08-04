use chrono::Utc;
use common::types::{Order, OrderStatus, OrderType, Side, Symbol, Trade};
use order_book::{OrderBook, RestingOrder};
use rust_decimal::Decimal;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

/// Result of submitting an order: any trades executed immediately,
/// plus the resulting status of the submitted order.
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub trades: Vec<Trade>,
    pub order_status: OrderStatus,
    pub remaining_quantity: Decimal,
}

#[derive(Debug)]
pub enum EngineCommand {
    Submit {
        order: Order,
        reply: oneshot::Sender<MatchResult>,
    },
    Cancel {
        order_id: Uuid,
        side: Side,
        price: Decimal,
        reply: oneshot::Sender<bool>,
    },
    Depth {
        levels: usize,
        reply: oneshot::Sender<order_book::BookDepth>,
    },
}

#[derive(Clone)]
pub struct EngineHandle {
    tx: mpsc::Sender<EngineCommand>,
}

impl EngineHandle {
    pub async fn submit(&self, order: Order) -> MatchResult {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(EngineCommand::Submit { order, reply }).await;
        rx.await.expect("engine task gone")
    }

    pub async fn cancel(&self, order_id: Uuid, side: Side, price: Decimal) -> bool {
        let (reply, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(EngineCommand::Cancel { order_id, side, price, reply })
            .await;
        rx.await.unwrap_or(false)
    }

    pub async fn depth(&self, levels: usize) -> order_book::BookDepth {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(EngineCommand::Depth { levels, reply }).await;
        rx.await.expect("engine task gone")
    }
}

/// Spawns one matching-engine actor task per symbol. Each owns its
/// OrderBook exclusively — no locking needed, commands are serialized
/// through the mpsc channel.
pub fn spawn(symbol: Symbol) -> EngineHandle {
    let (tx, mut rx) = mpsc::channel::<EngineCommand>(1024);
    let mut book = OrderBook::new(symbol);

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                EngineCommand::Submit { order, reply } => {
                    let result = match_order(&mut book, order);
                    let _ = reply.send(result);
                }
                EngineCommand::Cancel { order_id, side, price, reply } => {
                    let removed = book.cancel(side, price, order_id);
                    let _ = reply.send(removed);
                }
                EngineCommand::Depth { levels, reply } => {
                    let _ = reply.send(book.depth(levels));
                }
            }
        }
    });

    EngineHandle { tx }
}

fn opposite_side(side: Side) -> Side {
    match side {
        Side::Buy => Side::Sell,
        Side::Sell => Side::Buy,
    }
}

/// Core matching logic: crosses the incoming order against the
/// opposite side of the book at price-time priority until either
/// the order is fully filled or no more crossable liquidity remains.
/// Any remainder of a limit order rests on the book.
fn match_order(book: &mut OrderBook, mut order: Order) -> MatchResult {
    let mut trades = Vec::new();
    let contra_side = opposite_side(order.side);
    let mut remaining = order.quantity - order.filled_quantity;

    loop {
        if remaining <= Decimal::ZERO {
            break;
        }

        let best = book.peek_best(contra_side);
        let (best_price, resting) = match best {
            Some((p, o)) => (p, o.clone()),
            None => break,
        };

        // Check crossability
        let crosses = match order.order_type {
            OrderType::Market => true,
            OrderType::Limit => match order.side {
                Side::Buy => order.price.map_or(false, |p| p >= best_price),
                Side::Sell => order.price.map_or(false, |p| p <= best_price),
            },
        };

        if !crosses {
            break;
        }

        let fill_qty = remaining.min(resting.quantity);

        let (buy_order_id, sell_order_id) = match order.side {
            Side::Buy => (order.id, resting.id),
            Side::Sell => (resting.id, order.id),
        };

        trades.push(Trade {
            id: Uuid::new_v4(),
            symbol: order.symbol,
            buy_order_id,
            sell_order_id,
            price: best_price, // resting order's price — standard price-time priority rule
            quantity: fill_qty,
            executed_at: Utc::now(),
        });

        book.fill_best(contra_side, best_price, fill_qty);
        remaining -= fill_qty;
        order.filled_quantity += fill_qty;
    }

    // Rest any remainder of a limit order on the book.
    if remaining > Decimal::ZERO && order.order_type == OrderType::Limit {
        if let Some(price) = order.price {
            book.insert(
                order.side,
                price,
                RestingOrder {
                    id: order.id,
                    user_id: order.user_id,
                    quantity: remaining,
                },
            );
        }
    }

    let status = if remaining <= Decimal::ZERO {
        OrderStatus::Filled
    } else if order.filled_quantity > Decimal::ZERO {
        OrderStatus::PartiallyFilled
    } else if order.order_type == OrderType::Market {
        OrderStatus::Cancelled // market order with zero fill has no book presence
    } else {
        OrderStatus::Open
    };

    MatchResult {
        trades,
        order_status: status,
        remaining_quantity: remaining,
    }
}