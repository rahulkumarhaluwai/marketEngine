use crate::store::{Account, Alert, AlertDirection, Store};
use async_trait::async_trait;
use common::types::{Order, Symbol, Trade};
use risk_engine::AccountSnapshot;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct Inner {
    accounts: HashMap<Uuid, Account>,
    holdings: HashMap<Uuid, HashMap<Symbol, Decimal>>,
    orders: HashMap<Uuid, Order>,
    trades: Vec<Trade>,
    alerts: HashMap<Uuid, Alert>,
}

pub struct InMemoryStore {
    inner: Mutex<Inner>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self { inner: Mutex::new(Inner::default()) }
    }
}

#[async_trait]
impl Store for InMemoryStore {
    async fn create_account(&self, username: String, password_hash: String) -> Account {
        let account = Account { id: Uuid::new_v4(), username, cash_balance: Decimal::ZERO, password_hash };
        let mut inner = self.inner.lock().unwrap();
        inner.accounts.insert(account.id, account.clone());
        inner.holdings.insert(account.id, HashMap::new());
        account
    }

    async fn get_account(&self, user_id: Uuid) -> Option<Account> {
        self.inner.lock().unwrap().accounts.get(&user_id).cloned()
    }

    async fn get_account_by_username(&self, username: &str) -> Option<Account> {
        self.inner.lock().unwrap().accounts.values().find(|a| a.username == username).cloned()
    }

    async fn all_user_ids(&self) -> Vec<Uuid> {
        self.inner.lock().unwrap().accounts.keys().copied().collect()
    }

    async fn deposit(&self, user_id: Uuid, amount: Decimal) -> Option<Account> {
        let mut inner = self.inner.lock().unwrap();
        let account = inner.accounts.get_mut(&user_id)?;
        account.cash_balance += amount;
        Some(account.clone())
    }

    async fn account_snapshot(&self, user_id: Uuid) -> Option<AccountSnapshot> {
        let inner = self.inner.lock().unwrap();
        let account = inner.accounts.get(&user_id)?;
        let holdings = inner
            .holdings
            .get(&user_id)
            .map(|h| h.iter().map(|(s, q)| (s.as_str().to_string(), *q)).collect())
            .unwrap_or_default();

        let open_orders: Vec<&Order> = inner
            .orders
            .values()
            .filter(|o| {
                o.user_id == user_id
                    && matches!(o.status, common::types::OrderStatus::Open | common::types::OrderStatus::PartiallyFilled)
            })
            .collect();

        let reserved_cash = open_orders
            .iter()
            .filter(|o| o.side == common::types::Side::Buy)
            .map(|o| o.price.unwrap_or(Decimal::ZERO) * (o.quantity - o.filled_quantity))
            .sum();

        let mut reserved_assets: HashMap<String, Decimal> = HashMap::new();
        for o in open_orders.iter().filter(|o| o.side == common::types::Side::Sell) {
            *reserved_assets.entry(o.symbol.as_str().to_string()).or_insert(Decimal::ZERO) +=
                o.quantity - o.filled_quantity;
        }

        Some(AccountSnapshot {
            cash_balance: account.cash_balance,
            holdings,
            reserved_cash,
            reserved_assets,
            open_order_count: open_orders.len(),
        })
    }

    async fn holdings(&self, user_id: Uuid) -> Vec<(Symbol, Decimal)> {
        self.inner
            .lock()
            .unwrap()
            .holdings
            .get(&user_id)
            .map(|h| h.iter().map(|(s, q)| (*s, *q)).collect())
            .unwrap_or_default()
    }

    async fn save_order(&self, order: Order) {
        self.inner.lock().unwrap().orders.insert(order.id, order);
    }

    async fn update_order(&self, order: Order) {
        self.inner.lock().unwrap().orders.insert(order.id, order);
    }

    async fn orders_for_user(&self, user_id: Uuid) -> Vec<Order> {
        self.inner.lock().unwrap().orders.values().filter(|o| o.user_id == user_id).cloned().collect()
    }

    async fn save_trades(&self, trades: &[Trade]) {
        let mut inner = self.inner.lock().unwrap();

        for trade in trades {
            inner.trades.push(trade.clone());

            if let Some(buy_order) = inner.orders.get(&trade.buy_order_id).cloned() {
                *inner.holdings.entry(buy_order.user_id).or_default().entry(trade.symbol).or_insert(Decimal::ZERO) +=
                    trade.quantity;
                if let Some(acc) = inner.accounts.get_mut(&buy_order.user_id) {
                    acc.cash_balance -= trade.price * trade.quantity;
                }
            }

            if let Some(sell_order) = inner.orders.get(&trade.sell_order_id).cloned() {
                *inner.holdings.entry(sell_order.user_id).or_default().entry(trade.symbol).or_insert(Decimal::ZERO) -=
                    trade.quantity;
                if let Some(acc) = inner.accounts.get_mut(&sell_order.user_id) {
                    acc.cash_balance += trade.price * trade.quantity;
                }
            }
        }
    }

    async fn trades_for_user(&self, user_id: Uuid) -> Vec<Trade> {
        let inner = self.inner.lock().unwrap();
        inner
            .trades
            .iter()
            .filter(|t| {
                inner.orders.get(&t.buy_order_id).map_or(false, |o| o.user_id == user_id)
                    || inner.orders.get(&t.sell_order_id).map_or(false, |o| o.user_id == user_id)
            })
            .cloned()
            .collect()
    }

    async fn create_alert(
        &self,
        user_id: Uuid,
        symbol: Symbol,
        target_price: Decimal,
        direction: AlertDirection,
    ) -> Alert {
        let alert = Alert { id: Uuid::new_v4(), user_id, symbol, target_price, direction, triggered: false };
        self.inner.lock().unwrap().alerts.insert(alert.id, alert.clone());
        alert
    }

    async fn alerts_for_user(&self, user_id: Uuid) -> Vec<Alert> {
        self.inner.lock().unwrap().alerts.values().filter(|a| a.user_id == user_id).cloned().collect()
    }

    async fn untriggered_alerts_for_symbol(&self, symbol: Symbol) -> Vec<Alert> {
        self.inner
            .lock()
            .unwrap()
            .alerts
            .values()
            .filter(|a| a.symbol == symbol && !a.triggered)
            .cloned()
            .collect()
    }

    async fn mark_alert_triggered(&self, alert_id: Uuid) {
        if let Some(alert) = self.inner.lock().unwrap().alerts.get_mut(&alert_id) {
            alert.triggered = true;
        }
    }
}