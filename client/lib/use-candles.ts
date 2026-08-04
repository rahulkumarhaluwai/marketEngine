"use client";

import { useEffect, useState } from "react";
import { gqlClient } from "./graphql-client";
import { GET_CANDLES } from "./queries";

export type Candle = {
  bucketStart: string;
  open: string;
  high: string;
  low: string;
  close: string;
};

const SYMBOL_GQL_MAP: Record<string, string> = {
  "BTC-USD": "BTC_USD",
  "ETH-USD": "ETH_USD",
  AAPL: "AAPL",
  TSLA: "TSLA",
  GOOGL: "GOOGL",
  MSFT: "MSFT",
  AMZN: "AMZN",
};

export function useCandles(wsSymbol: string, limit: number = 100) {
  const [candles, setCandles] = useState<Candle[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);

    const symbolGql = SYMBOL_GQL_MAP[wsSymbol] ?? wsSymbol;

    gqlClient
      .request<{ candles: Candle[] }>(GET_CANDLES, { symbol: symbolGql, limit })
      .then((data) => {
        if (!cancelled) setCandles(data.candles);
      })
      .catch(() => {
        if (!cancelled) setCandles([]);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [wsSymbol, limit]);

  return { candles, loading };
}