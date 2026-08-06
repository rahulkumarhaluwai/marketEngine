use common::types::{Order, OrderType, Side};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskError {
    InsufficientBalance,
    InsufficientAssetBalance,
    InvalidQuantity,
    InvalidPrice,
    OrderSizeExceedsLimit,
    ExceedsOpenOrderLimit,
}

#[derive(Debug, Clone)]
pub struct AccountSnapshot {
    pub cash_balance: Decimal,
    pub holdings: HashMap<String, Decimal>,
    pub reserved_cash: Decimal,
    pub reserved_assets: HashMap<String, Decimal>,
    pub open_order_count: usize,
}

pub struct RiskConfig {
    pub max_order_notional: Decimal,
    pub max_open_orders_per_user: usize,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_order_notional: Decimal::new(1_000_000, 0),
            max_open_orders_per_user: 50,
        }
    }
}

pub struct RiskEngine {
    config: RiskConfig,
}

impl RiskEngine {
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }

    pub fn validate(
        &self,
        order: &Order,
        account: &AccountSnapshot,
        reference_price: Decimal,
    ) -> Result<(), RiskError> {
        if order.quantity <= Decimal::ZERO {
            return Err(RiskError::InvalidQuantity);
        }

        if order.order_type == OrderType::Limit {
            match order.price {
                Some(p) if p > Decimal::ZERO => {}
                _ => return Err(RiskError::InvalidPrice),
            }
        }

        if account.open_order_count >= self.config.max_open_orders_per_user {
            return Err(RiskError::ExceedsOpenOrderLimit);
        }

        let effective_price = order.price.unwrap_or(reference_price);
        let notional = effective_price * order.quantity;

        if notional > self.config.max_order_notional {
            return Err(RiskError::OrderSizeExceedsLimit);
        }

        match order.side {
            Side::Buy => {
                let available_cash = account.cash_balance - account.reserved_cash;
                if notional > available_cash {
                    return Err(RiskError::InsufficientBalance);
                }
            }
            Side::Sell => {
                let symbol_key = order.symbol.as_str();
                let held = account
                    .holdings
                    .get(symbol_key)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let reserved = account
                    .reserved_assets
                    .get(symbol_key)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let available = held - reserved;
                if order.quantity > available {
                    return Err(RiskError::InsufficientAssetBalance);
                }
            }
        }

        Ok(())
    }
}

pub fn reserve_for_order(order: &Order, reference_price: Decimal) -> Reservation {
    match order.side {
        Side::Buy => {
            let price = order.price.unwrap_or(reference_price);
            Reservation::Cash(price * order.quantity)
        }
        Side::Sell => Reservation::Asset(order.symbol.as_str().to_string(), order.quantity),
    }
}

pub enum Reservation {
    Cash(Decimal),
    Asset(String, Decimal),
}

pub type UserId = Uuid;