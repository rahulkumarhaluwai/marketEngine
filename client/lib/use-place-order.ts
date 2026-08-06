"use client";

import { useState } from "react";
import { gqlClient } from "./graphql-client";
import { PLACE_ORDER } from "./queries";

type OrderInput = {
  userId: string;
  symbol: string;
  side: "BUY" | "SELL";
  orderType: "MARKET" | "LIMIT";
  quantity: string;
  price?: string;
};

type OrderResult = {
  id: string;
  symbol: string;
  side: string;
  orderType: string;
  price: string | null;
  quantity: string;
  filledQuantity: string;
  status: string;
};

export function usePlaceOrder() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<OrderResult | null>(null);

  async function placeOrder(input: OrderInput) {
    setLoading(true);
    setError(null);
    try {
      const data = await gqlClient.request<{ placeOrder: OrderResult }>(PLACE_ORDER, input);
      setResult(data.placeOrder);
      return data.placeOrder;
    } catch (err: any) {
      const message = err?.response?.errors?.[0]?.message ?? "Failed to place order";
      setError(message);
      return null;
    } finally {
      setLoading(false);
    }
  }

  return { placeOrder, loading, error, result };
}