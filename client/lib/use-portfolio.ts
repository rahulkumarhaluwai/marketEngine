"use client";

import { useEffect, useState, useCallback } from "react";
import { gqlClient } from "./graphql-client";
import { GET_PORTFOLIO } from "./queries";

export type Position = {
  symbol: string;
  quantity: string;
  avgCost: string;
  marketValue: string;
  unrealizedPnl: string;
};

export type Portfolio = {
  cashBalance: string;
  totalMarketValue: string;
  totalUnrealizedPnl: string;
  positions: Position[];
};

export function usePortfolio(userId: string | null) {
  const [portfolio, setPortfolio] = useState<Portfolio | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!userId) return;
    setLoading(true);
    setError(null);
    try {
      const data = await gqlClient.request<{ portfolio: Portfolio }>(GET_PORTFOLIO, { userId });
      setPortfolio(data.portfolio);
    } catch (err: any) {
      setError(err?.response?.errors?.[0]?.message ?? "Failed to load portfolio");
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { portfolio, loading, error, refresh };
}