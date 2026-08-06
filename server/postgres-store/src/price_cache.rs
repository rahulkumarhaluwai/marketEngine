use common::types::Symbol;
use redis::AsyncCommands;
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Clone)]
pub struct PriceCache {
    client: redis::Client,
}

impl PriceCache {
    pub fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn set_price(&self, symbol: Symbol, price: Decimal) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("marketengine:price:{}", symbol.as_str());
        let _: () = conn.set(key, price.to_string()).await?;
        Ok(())
    }

    pub async fn get_price(&self, symbol: Symbol) -> anyhow::Result<Option<Decimal>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("marketengine:price:{}", symbol.as_str());
        let raw: Option<String> = conn.get(key).await?;
        Ok(raw.and_then(|s| Decimal::from_str(&s).ok()))
    }
}