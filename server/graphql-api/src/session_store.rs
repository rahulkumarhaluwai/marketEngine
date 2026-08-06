use rand::RngCore;
use redis::AsyncCommands;
use uuid::Uuid;

const SESSION_TTL_SECS: u64 = 60 * 60 * 24 * 7;
const KEY_PREFIX: &str = "marketengine:";

#[derive(Clone)]
pub struct SessionStore {
    client: redis::Client,
}

impl SessionStore {
    pub fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    fn generate_token() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    pub async fn create_session(&self, user_id: Uuid) -> anyhow::Result<String> {
        let token = Self::generate_token();
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("{KEY_PREFIX}session:{token}");
        let _: () = conn.set_ex(key, user_id.to_string(), SESSION_TTL_SECS).await?;
        Ok(token)
    }

    pub async fn resolve(&self, token: &str) -> anyhow::Result<Option<Uuid>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("{KEY_PREFIX}session:{token}");
        let raw: Option<String> = conn.get(&key).await?;
        Ok(raw.and_then(|s| Uuid::parse_str(&s).ok()))
    }

    pub async fn destroy(&self, token: &str) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("{KEY_PREFIX}session:{token}");
        let _: () = conn.del(key).await?;
        Ok(())
    }
}