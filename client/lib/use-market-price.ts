"use client";

import { useEffect, useState } from "react";
import { getWsClient } from "./ws-client";

export type PricePoint = {
  price: string;
  timestamp: string;
  direction: "up" | "down" | "flat";
};

export function useMarketPrice(symbol: string) {
  const channel = `market:${symbol}`;
  const [data, setData] = useState<PricePoint | null>(null);

  useEffect(() => {
    const client = getWsClient();
    let lastPrice: number | null = null;

    const unsubscribe = client.subscribe(channel, (tick) => {
      const priceNum = parseFloat(tick.price);
      const direction: PricePoint["direction"] =
        lastPrice === null ? "flat" : priceNum > lastPrice ? "up" : priceNum < lastPrice ? "down" : "flat";
      lastPrice = priceNum;

      setData({ price: tick.price, timestamp: tick.timestamp, direction });
    });

    return unsubscribe;
  }, [channel]);

  return data;
}