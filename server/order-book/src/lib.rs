use common::types::{Order, Side, Symbol};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, VecDeque};
use uuid::Uuid;

/// A resting order at a price level, with insertion order preserved
/// for time priority.
#[derive(Debug, Clone)]
pub struct RestingOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quantity: Decimal, // remaining, unfilled quantity
}

impl From<&Order> for RestingOrder {
    fn from(o: &Order) -> Self {
        Self {
            id: o.id,
            user_id: o.user_id,
            quantity: o.quantity - o.filled_quantity,
        }
    }
}

/// Price-time priority order book for a single symbol.
///
/// Bids: highest price first (reverse ordering).
/// Asks: lowest price first (natural ordering).
pub struct OrderBook {
    pub symbol: Symbol,
    pub bids: BTreeMap<Decimal, VecDeque<RestingOrder>>, // iterate .rev() for best-first
    pub asks: BTreeMap<Decimal, VecDeque<RestingOrder>>, // iterate normally for best-first
}

impl OrderBook {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn book_mut(&mut self, side: Side) -> &mut BTreeMap<Decimal, VecDeque<RestingOrder>> {
        match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        }
    }

    /// Insert a resting limit order onto the book (time priority = push_back).
    pub fn insert(&mut self, side: Side, price: Decimal, order: RestingOrder) {
        self.book_mut(side)
            .entry(price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    /// Best bid price (highest).
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().copied()
    }

    /// Best ask price (lowest).
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().copied()
    }

    /// Remove an order by id and price, wherever it sits in the queue.
    /// Returns true if found and removed. Cleans up empty price levels.
    pub fn cancel(&mut self, side: Side, price: Decimal, order_id: Uuid) -> bool {
        let book = self.book_mut(side);
        if let Some(queue) = book.get_mut(&price) {
            let before = queue.len();
            queue.retain(|o| o.id != order_id);
            let removed = queue.len() != before;
            if queue.is_empty() {
                book.remove(&price);
            }
            return removed;
        }
        false
    }

    /// Peek the front (oldest, first priority) order of the best price level
    /// on the given side, without removing it.
    pub fn peek_best(&mut self, side: Side) -> Option<(Decimal, &RestingOrder)> {
        match side {
            Side::Buy => {
                let price = self.bids.keys().next_back().copied()?;
                self.bids.get(&price).and_then(|q| q.front()).map(|o| (price, o))
            }
            Side::Sell => {
                let price = self.asks.keys().next().copied()?;
                self.asks.get(&price).and_then(|q| q.front()).map(|o| (price, o))
            }
        }
    }

    /// Reduce or remove the front order of the best price level after a fill.
    pub fn fill_best(&mut self, side: Side, price: Decimal, filled_qty: Decimal) {
        let book = self.book_mut(side);
        if let Some(queue) = book.get_mut(&price) {
            if let Some(front) = queue.front_mut() {
                front.quantity -= filled_qty;
                if front.quantity <= Decimal::ZERO {
                    queue.pop_front();
                }
            }
            if queue.is_empty() {
                book.remove(&price);
            }
        }
    }

    /// Snapshot depth for display/UI (top N levels each side).
    pub fn depth(&self, levels: usize) -> BookDepth {
        let asks: Vec<(Decimal, Decimal)> = self
            .asks
            .iter()
            .take(levels)
            .map(|(price, q)| (*price, q.iter().map(|o| o.quantity).sum()))
            .collect();

        let bids: Vec<(Decimal, Decimal)> = self
            .bids
            .iter()
            .rev()
            .take(levels)
            .map(|(price, q)| (*price, q.iter().map(|o| o.quantity).sum()))
            .collect();

        BookDepth { asks, bids }
    }
}

#[derive(Debug, Clone)]
pub struct BookDepth {
    pub asks: Vec<(Decimal, Decimal)>, // ascending price
    pub bids: Vec<(Decimal, Decimal)>, // descending price
}