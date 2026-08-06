use chrono::Utc;
use common::types::{Order, OrderStatus, OrderType, Side, Symbol};
use graphql_api::store::Store;
use matching_engine::EngineHandle;
use rand::Rng;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct MarketMakerConfig {
    pub levels: usize,
    pub level_spacing: Decimal,
    pub level_quantity: Decimal,
    pub refresh_interval_secs: u64,
}

impl Default for MarketMakerConfig {
    fn default() -> Self {
        Self {
            levels: 5,
            level_spacing: Decimal::from_str("5.0").unwrap(),
            level_quantity: Decimal::from_str("0.5").unwrap(),
            refresh_interval_secs: 5,
        }
    }
}
async fn ensure_bot_account(store: &Arc<dyn Store>, username: &str) -> Uuid {
    if let Some(existing) = store.get_account_by_username(username).await {
        return existing.id;
    }
    let account = store.create_account(username.to_string(), "bot".to_string()).await;
    store.deposit(account.id, Decimal::from(10_000_000)).await;
    account.id
}

pub fn spawn(
    symbol: Symbol,
    store: Arc<dyn Store>,
    engine: EngineHandle,
    reference_prices: Arc<tokio::sync::RwLock<HashMap<Symbol, Decimal>>>,
    config: MarketMakerConfig,
) {
    tokio::spawn(async move {
        let bot_username = format!("bot_maker_{}", symbol.as_str());
        let bot_id = ensure_bot_account(&store, &bot_username).await;
        tracing::info!("market-maker bot ready for {} (account {})", symbol.as_str(), bot_id);

        loop {
            let reference_price = reference_prices.read().await.get(&symbol).copied();

            if let Some(mid_price) = reference_price {
                place_level_orders(&store, &engine, bot_id, symbol, mid_price, &config).await;
            }

            tokio::time::sleep(Duration::from_secs(config.refresh_interval_secs)).await;
        }
    });
}

async fn place_level_orders(
    store: &Arc<dyn Store>,
    engine: &EngineHandle,
    bot_id: Uuid,
    symbol: Symbol,
    mid_price: Decimal,
    config: &MarketMakerConfig,
) {
    for i in 1..=config.levels {
        let offset = config.level_spacing * Decimal::from(i as i64);
        let jitter = {
            let mut rng = rand::thread_rng();
            Decimal::from(rng.gen_range(-2..=2))
        };

        let bid_price = mid_price - offset + jitter;
        let ask_price = mid_price + offset + jitter;

        place_resting_order(store, engine, bot_id, symbol, Side::Buy, bid_price, config.level_quantity).await;
        place_resting_order(store, engine, bot_id, symbol, Side::Sell, ask_price, config.level_quantity).await;
    }
}

async fn place_resting_order(
    store: &Arc<dyn Store>,
    engine: &EngineHandle,
    bot_id: Uuid,
    symbol: Symbol,
    side: Side,
    price: Decimal,
    quantity: Decimal,
) {
    if price <= Decimal::ZERO {
        return;
    }

    let order = Order {
        id: Uuid::new_v4(),
        user_id: bot_id,
        symbol,
        side,
        order_type: OrderType::Limit,
        price: Some(price),
        quantity,
        filled_quantity: Decimal::ZERO,
        status: OrderStatus::Open,
        created_at: Utc::now(),
    };

    store.save_order(order.clone()).await;

    let result = engine.submit(order.clone()).await;

    let mut updated = order.clone();
    updated.filled_quantity = order.quantity - result.remaining_quantity;
    updated.status = result.order_status;
    store.update_order(updated).await;

    if !result.trades.is_empty() {
        store.save_trades(&result.trades).await;
    }
}