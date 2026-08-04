use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Symbol {
    BtcUsd,
    EthUsd,
}

impl Symbol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Symbol::BtcUsd => "BTC-USD",
            Symbol::EthUsd => "ETH-USD",
        }
    }

    pub fn channel(&self) -> String {
        format!("market:{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub symbol: Symbol,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Option<Decimal>, // None for market orders
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggeredEvent {
    pub alert_id: Uuid,
    pub user_id: Uuid,
    pub symbol: Symbol,
    pub target_price: Decimal,
    pub direction: AlertDirectionEvent,
    pub price_at_trigger: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertDirectionEvent {
    Above,
    Below,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub symbol: Symbol,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: Decimal,
    pub quantity: Decimal,
    pub executed_at: DateTime<Utc>,
}