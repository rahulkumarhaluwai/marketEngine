use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

pub struct CreditPack {
    pub id: &'static str,
    pub label: &'static str,
    pub usd_cents: i64,
    pub virtual_credits: i64,
}

pub const CREDIT_PACKS: [CreditPack; 3] = [
    CreditPack { id: "starter", label: "Starter Pack", usd_cents: 100, virtual_credits: 10_000 },
    CreditPack { id: "trader", label: "Trader Pack", usd_cents: 500, virtual_credits: 60_000 },
    CreditPack { id: "pro", label: "Pro Pack", usd_cents: 1000, virtual_credits: 150_000 },
];

pub async fn create_checkout_session(
    secret_key: &str,
    pack: &CreditPack,
    user_id: Uuid,
    success_url: &str,
    cancel_url: &str,
) -> anyhow::Result<String> {
    let user_id_str = user_id.to_string();
    let usd_cents_str = pack.usd_cents.to_string();

    let params = [
        ("mode", "payment"),
        ("success_url", success_url),
        ("cancel_url", cancel_url),
        ("client_reference_id", &user_id_str),
        ("metadata[user_id]", &user_id_str),
        ("metadata[pack_id]", pack.id),
        ("line_items[0][quantity]", "1"),
        ("line_items[0][price_data][currency]", "usd"),
        ("line_items[0][price_data][unit_amount]", &usd_cents_str),
        ("line_items[0][price_data][product_data][name]", pack.label),
    ];

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .bearer_auth(secret_key)
        .form(&params)
        .send()
        .await?;

    let status = resp.status();
    let body: serde_json::Value = resp.json().await?;

    if !status.is_success() {
        anyhow::bail!("stripe error: {}", body);
    }

    body["url"].as_str().map(|s| s.to_string()).ok_or_else(|| anyhow::anyhow!("no checkout url in stripe response"))
}

pub fn verify_webhook_signature(payload: &[u8], sig_header: &str, webhook_secret: &str) -> bool {
    let mut timestamp = None;
    let mut v1_sig = None;

    for part in sig_header.split(',') {
        let mut kv = part.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("t"), Some(v)) => timestamp = Some(v),
            (Some("v1"), Some(v)) => v1_sig = Some(v),
            _ => {}
        }
    }

    let (Some(t), Some(sig)) = (timestamp, v1_sig) else { return false };
    let signed_payload = format!("{}.{}", t, String::from_utf8_lossy(payload));

    type HmacSha256 = Hmac<Sha256>;
    let Ok(mut mac) = HmacSha256::new_from_slice(webhook_secret.as_bytes()) else { return false };
    mac.update(signed_payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    expected == sig
}