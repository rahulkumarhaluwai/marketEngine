
CREATE TABLE IF NOT EXISTS accounts (
    user_id      UUID PRIMARY KEY,
    username     TEXT NOT NULL UNIQUE,
    cash_balance NUMERIC(20,8) NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS holdings (
    user_id  UUID NOT NULL REFERENCES accounts(user_id),
    symbol   TEXT NOT NULL,
    quantity NUMERIC(20,8) NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, symbol)
);

CREATE TABLE IF NOT EXISTS orders (
    order_id         UUID PRIMARY KEY,
    user_id          UUID NOT NULL REFERENCES accounts(user_id),
    symbol           TEXT NOT NULL,
    side             TEXT NOT NULL,
    order_type       TEXT NOT NULL,
    price            NUMERIC(20,8),
    quantity         NUMERIC(20,8) NOT NULL,
    filled_quantity  NUMERIC(20,8) NOT NULL DEFAULT 0,
    status           TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_orders_user ON orders (user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS trades (
    trade_id      UUID PRIMARY KEY,
    symbol        TEXT NOT NULL,
    buy_order_id  UUID NOT NULL,
    sell_order_id UUID NOT NULL,
    price         NUMERIC(20,8) NOT NULL,
    quantity      NUMERIC(20,8) NOT NULL,
    executed_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_trades_buy ON trades (buy_order_id);
CREATE INDEX IF NOT EXISTS idx_trades_sell ON trades (sell_order_id);

CREATE TABLE IF NOT EXISTS order_events (
    event_id   UUID PRIMARY KEY,
    order_id   UUID NOT NULL,
    event_type TEXT NOT NULL,
    detail     TEXT,
    event_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_order_events_order ON order_events (order_id, event_at DESC);

CREATE TABLE IF NOT EXISTS alerts (
    alert_id     UUID PRIMARY KEY,
    user_id      UUID NOT NULL REFERENCES accounts(user_id),
    symbol       TEXT NOT NULL,
    target_price NUMERIC(20,8) NOT NULL,
    direction    TEXT NOT NULL,
    triggered    BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS market_ticks (
    symbol TEXT NOT NULL,
    ts     TIMESTAMPTZ NOT NULL,
    price  NUMERIC(20,8) NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ticks_symbol_ts ON market_ticks (symbol, ts DESC);

ALTER TABLE accounts ADD COLUMN IF NOT EXISTS password_hash TEXT NOT NULL DEFAULT '';
-- Time-series tick table, converted to a TimescaleDB hypertable below.
CREATE TABLE IF NOT EXISTS market_ticks (
    symbol TEXT NOT NULL,
    ts     TIMESTAMPTZ NOT NULL,
    price  NUMERIC(20,8) NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ticks_symbol_ts ON market_ticks (symbol, ts DESC);