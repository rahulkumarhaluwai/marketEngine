use common::types::{Order, Side, Symbol};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, VecDeque};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RestingOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quantity: Decimal,
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

pub struct OrderBook {
    pub symbol: Symbol,
    pub bids: BTreeMap<Decimal, VecDeque<RestingOrder>>,
    pub asks: BTreeMap<Decimal, VecDeque<RestingOrder>>,
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

    pub fn insert(&mut self, side: Side, price: Decimal, order: RestingOrder) {
        self.book_mut(side)
            .entry(price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().copied()
    }

    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().copied()
    }

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
    pub asks: Vec<(Decimal, Decimal)>,
    pub bids: Vec<(Decimal, Decimal)>,
}