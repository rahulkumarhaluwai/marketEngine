use async_trait::async_trait;
use common::types::{Order, Symbol, Trade};
use risk_engine::AccountSnapshot;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub cash_balance: Decimal,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: Symbol,
    pub target_price: Decimal,
    pub direction: AlertDirection,
    pub triggered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertDirection {
    Above,
    Below,
}

#[async_trait]
pub trait Store: Send + Sync {
    async fn create_account(&self, username: String, password_hash: String) -> Account;
    async fn get_account(&self, user_id: Uuid) -> Option<Account>;
    async fn get_account_by_username(&self, username: &str) -> Option<Account>;
    async fn deposit(&self, user_id: Uuid, amount: Decimal) -> Option<Account>;

    async fn account_snapshot(&self, user_id: Uuid) -> Option<AccountSnapshot>;
    async fn holdings(&self, user_id: Uuid) -> Vec<(Symbol, Decimal)>;

    async fn save_order(&self, order: Order);
    async fn update_order(&self, order: Order);
    async fn orders_for_user(&self, user_id: Uuid) -> Vec<Order>;

    async fn save_trades(&self, trades: &[Trade]);
    async fn trades_for_user(&self, user_id: Uuid) -> Vec<Trade>;

    async fn create_alert(
        &self,
        user_id: Uuid,
        symbol: Symbol,
        target_price: Decimal,
        direction: AlertDirection,
    ) -> Alert;
    async fn alerts_for_user(&self, user_id: Uuid) -> Vec<Alert>;
    async fn untriggered_alerts_for_symbol(&self, symbol: Symbol) -> Vec<Alert>;
    async fn mark_alert_triggered(&self, alert_id: Uuid);
}