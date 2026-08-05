use crate::store::Store;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
struct BillingState {
    store: Arc<dyn Store>,
    webhook_secret: String,
}

async fn stripe_webhook(State(state): State<BillingState>, headers: HeaderMap, body: Bytes) -> StatusCode {
    let sig = headers.get("stripe-signature").and_then(|v| v.to_str().ok()).unwrap_or("");

    if !billing::verify_webhook_signature(&body, sig, &state.webhook_secret) {
        tracing::warn!("stripe webhook signature verification failed");
        return StatusCode::BAD_REQUEST;
    }

    let Ok(event): Result<serde_json::Value, _> = serde_json::from_slice(&body) else {
        return StatusCode::BAD_REQUEST;
    };

    if event["type"] == "checkout.session.completed" {
        let obj = &event["data"]["object"];
        let user_id_str = obj["metadata"]["user_id"].as_str();
        let pack_id = obj["metadata"]["pack_id"].as_str();

        if let (Some(uid), Some(pid)) = (user_id_str, pack_id) {
            if let Ok(user_id) = Uuid::parse_str(uid) {
                if let Some(pack) = billing::CREDIT_PACKS.iter().find(|p| p.id == pid) {
                    let amount = Decimal::from(pack.virtual_credits);
                    if state.store.deposit(user_id, amount).await.is_some() {
                        tracing::info!("credited {} virtual dollars to user {}", amount, user_id);
                    }
                }
            }
        }
    }

    StatusCode::OK
}

pub fn billing_router(store: Arc<dyn Store>, webhook_secret: String) -> Router {
    Router::new()
        .route("/stripe/webhook", post(stripe_webhook))
        .with_state(BillingState { store, webhook_secret })
}