use redis::AsyncCommands;
use uuid::Uuid;

const LEADERBOARD_KEY: &str = "marketengine:leaderboard:equity";

#[derive(Clone)]
pub struct Leaderboard {
    client: redis::Client,
}

impl Leaderboard {
    pub fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    /// Sets a user's current equity score. Called periodically by a
    /// background task for every account.
    pub async fn set_equity(&self, user_id: Uuid, equity: f64) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.zadd(LEADERBOARD_KEY, user_id.to_string(), equity).await?;
        Ok(())
    }

    /// Returns the top N (user_id, equity) pairs, highest first.
    pub async fn top(&self, limit: isize) -> anyhow::Result<Vec<(Uuid, f64)>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let raw: Vec<(String, f64)> = conn.zrevrange_withscores(LEADERBOARD_KEY, 0, limit - 1).await?;
        Ok(raw
            .into_iter()
            .filter_map(|(id, score)| Uuid::parse_str(&id).ok().map(|uuid| (uuid, score)))
            .collect())
    }
}