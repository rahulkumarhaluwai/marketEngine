"use client";

import { useEffect } from "react";
import { getWsClient } from "./ws-client";

type AlertTriggerMessage = {
  channel: string;
  alert_id: string;
  symbol: string;
  target_price: string;
  price_at_trigger: string;
  direction: string;
};

export function useAlertNotifications(userId: string | null, onTrigger: (msg: AlertTriggerMessage) => void) {
  useEffect(() => {
    if (!userId) return;
    const client = getWsClient();
    const channel = `alerts:${userId}`;

    const unsubscribe = client.subscribe(channel, (raw) => {
      onTrigger(raw as unknown as AlertTriggerMessage);
    });

    return unsubscribe;
  }, [userId, onTrigger]);
}