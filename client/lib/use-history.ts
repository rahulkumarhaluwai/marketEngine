"use client";

import { useEffect, useState } from "react";
import { gqlClient } from "./graphql-client";
import { GET_ORDER_HISTORY, GET_TRADE_HISTORY } from "./queries";

export type OrderRecord = {
  id: string;
  symbol: string;
  side: string;
  orderType: string;
  price: string | null;
  quantity: string;
  filledQuantity: string;
  status: string;
};

export type TradeRecord = {
  id: string;
  symbol: string;
  price: string;
  quantity: string;
  executedAt: string;
};

export function useOrderHistory(userId: string | null) {
  const [orders, setOrders] = useState<OrderRecord[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!userId) return;
    setLoading(true);
    gqlClient
      .request<{ orderHistory: OrderRecord[] }>(GET_ORDER_HISTORY, { userId })
      .then((data) => setOrders(data.orderHistory))
      .finally(() => setLoading(false));
  }, [userId]);

  return { orders, loading };
}

export function useTradeHistory(userId: string | null) {
  const [trades, setTrades] = useState<TradeRecord[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!userId) return;
    setLoading(true);
    gqlClient
      .request<{ tradeHistory: TradeRecord[] }>(GET_TRADE_HISTORY, { userId })
      .then((data) => setTrades(data.tradeHistory))
      .finally(() => setLoading(false));
  }, [userId]);

  return { trades, loading };
}