# marketEngine — Real-Time Simulated Trading Platform

A fully self-contained, simulated stock and crypto trading platform built to demonstrate real-time systems, concurrent order matching, and full-stack engineering. No real money, no real brokers — everything is simulated end-to-end.

---

## Features

- **Account system** — registration, login, Redis-backed session auth (Argon2 password hashing, no JWT)
- **Virtual deposits** — Stripe Checkout in test mode (no real charges), credit packs converted to virtual trading balance
- **Live market data** — 7 simulated assets (BTC/USD, ETH/USD, AAPL, TSLA, GOOGL, MSFT, AMZN), streamed over WebSocket
- **Order types** — market and limit orders, price-time priority matching
- **Order book** — custom-built, per-symbol, price-time priority matching engine running as independent concurrent actors
- **Market-maker bots** — background liquidity providers keep the book populated on every symbol
- **Portfolio & P&L** — live-updating positions, unrealized P&L, cash balance
- **Order & trade history** — full audit trail per user
- **Price alerts** — set target price alerts, delivered live via WebSocket when triggered
- **Leaderboard** — Redis sorted-set ranking by account equity, updated continuously
- **Rate limiting** — Redis-backed limits on order placement and login attempts
- **Candlestick charts** — live OHLC charts per asset, built from persisted tick data
- **Dark/light theme** — persisted, system-wide toggle

---

## Tech Stack

**Backend (Rust)**
- Tokio (async runtime, actor-per-symbol concurrency)
- Axum (HTTP/WebSocket server)
- async-graphql (GraphQL API)
- sqlx (Postgres)
- redis-rs (sessions, leaderboard, rate limiting, price cache)
- Argon2 (password hashing)

**Frontend**
- Next.js (App Router)
- Tailwind CSS v4
- graphql-request
- lightweight-charts (candlestick charts)

**Infrastructure**
- Postgres — [Neon](https://neon.tech)
- Redis — [Redis Cloud](https://redis.io)
- Backend hosting — [Render](https://render.com)
- Frontend hosting — [Vercel](https://vercel.com)
- Payments — [Stripe](https://stripe.com) (test mode only)
- CI — GitHub Actions
