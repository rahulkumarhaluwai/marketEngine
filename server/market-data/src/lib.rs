use chrono::Utc;
use common::types::{Symbol, Tick};
use rand::Rng;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct MarketDataFeed {
    sender: broadcast::Sender<Tick>,
}

impl MarketDataFeed {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Tick> {
        self.sender.subscribe()
    }
}

fn starting_price(symbol: Symbol) -> Decimal {
    match symbol {
        Symbol::BtcUsd => Decimal::from_str("105230.00").unwrap(),
        Symbol::EthUsd => Decimal::from_str("3450.00").unwrap(),
        Symbol::Aapl => Decimal::from_str("227.50").unwrap(),
        Symbol::Tsla => Decimal::from_str("245.80").unwrap(),
        Symbol::Googl => Decimal::from_str("178.20").unwrap(),
        Symbol::Msft => Decimal::from_str("430.10").unwrap(),
        Symbol::Amzn => Decimal::from_str("195.40").unwrap(),
    }
}

pub async fn run(feed: MarketDataFeed) {
    let mut handles = Vec::new();

    for symbol in Symbol::all() {
        let feed = feed.clone();
        let handle = tokio::spawn(async move {
            let mut price = starting_price(symbol);

            loop {
                let (direction, magnitude) = {
                    let mut rng = rand::thread_rng();
                    let direction: i32 = rng.gen_range(-1..=1);
                    let magnitude: i64 = rng.gen_range(1..=50);
                    (direction, magnitude)
                };

                let tick_size = match symbol {
                    Symbol::BtcUsd | Symbol::EthUsd => Decimal::from_str("0.01").unwrap(),
                    _ => Decimal::from_str("0.005").unwrap(),
                };

                let step = tick_size * Decimal::from(direction) * Decimal::from(magnitude);
                price += step;
                if price <= Decimal::ZERO {
                    price = starting_price(symbol);
                }

                let tick = Tick { symbol, price, timestamp: Utc::now() };
                if feed.sender.send(tick).is_err() {
                    tracing::warn!("no subscribers for {}", symbol.as_str());
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
        handles.push(handle);
    }

    futures_wait_all(handles).await;
}

async fn futures_wait_all(handles: Vec<tokio::task::JoinHandle<()>>) {
    for h in handles {
        let _ = h.await;
    }
}