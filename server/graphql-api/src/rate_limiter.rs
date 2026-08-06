use redis::AsyncCommands;

#[derive(Clone)]
pub struct RateLimiter {
    client: redis::Client,
}

impl RateLimiter {
    pub fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn check_and_increment(&self, key: &str, max: i64, window_secs: u64) -> anyhow::Result<bool> {
    let key = format!("marketengine:{key}");
    let mut conn = self.client.get_multiplexed_async_connection().await?;
    let count: i64 = conn.incr(&key, 1).await?;

        if count == 1 {
            let _: () = conn.expire(&key, window_secs as i64).await?;
        }

        Ok(count <= max)
    }
}