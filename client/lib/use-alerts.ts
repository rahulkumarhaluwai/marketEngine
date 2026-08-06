"use client";

import { useEffect, useState, useCallback } from "react";
import { gqlClient } from "./graphql-client";
import { GET_ALERTS, CREATE_ALERT } from "./queries";

export type AlertRecord = {
  id: string;
  symbol: string;
  targetPrice: string;
  direction: string;
  triggered: boolean;
};

type CreateAlertInput = {
  userId: string;
  symbol: string;
  targetPrice: string;
  direction: "ABOVE" | "BELOW";
};

export function useAlerts(userId: string | null) {
  const [alerts, setAlerts] = useState<AlertRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!userId) return;
    setLoading(true);
    try {
      const data = await gqlClient.request<{ alerts: AlertRecord[] }>(GET_ALERTS, { userId });
      setAlerts(data.alerts);
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  async function createAlert(input: CreateAlertInput) {
    setError(null);
    try {
      const data = await gqlClient.request<{ createAlert: AlertRecord }>(CREATE_ALERT, input);
      setAlerts((prev) => [...prev, data.createAlert]);
      return data.createAlert;
    } catch (err: any) {
      setError(err?.response?.errors?.[0]?.message ?? "Failed to create alert");
      return null;
    }
  }

  return { alerts, loading, error, refresh, createAlert };
}