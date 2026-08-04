"use client";

import { useCallback, useState } from "react";
import { useSession } from "@/app/providers";
import { useAlerts } from "@/lib/use-alerts";
import { useAlertNotifications } from "@/lib/use-alert-notifications";
import { AlertForm } from "@/components/AlertForm";
import { AlertList } from "@/components/AlertList";

export default function AlertsPage() {
  const { userId } = useSession();
  const { alerts, loading, refresh } = useAlerts(userId);
  const [notice, setNotice] = useState<string | null>(null);

  const handleTrigger = useCallback(
    (msg: { symbol: string; direction: string; price_at_trigger: string }) => {
      setNotice(`${msg.symbol} crossed ${msg.direction.toLowerCase()} — now $${Number(msg.price_at_trigger).toFixed(2)}`);
      refresh();
    },
    [refresh]
  );

  useAlertNotifications(userId, handleTrigger);

  if (!userId) {
    return (
      <main className="min-h-screen bg-gray-950 text-white p-8">
        <p className="text-gray-400">Log in to manage alerts.</p>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-gray-950 text-white p-8">
      <h1 className="text-2xl font-semibold mb-6">Price Alerts</h1>

      {notice && (
        <div className="mb-6 rounded-lg border border-yellow-700 bg-yellow-900/30 px-4 py-3 text-yellow-300 text-sm">
          {notice}
        </div>
      )}

      <div className="grid grid-cols-2 gap-12">
        <div>
          <h2 className="text-lg font-medium mb-4">New Alert</h2>
          <AlertForm onCreated={refresh} />
        </div>
        <div>
          <h2 className="text-lg font-medium mb-4">Your Alerts</h2>
          {loading ? <p className="text-gray-400">Loading...</p> : <AlertList alerts={alerts} />}
        </div>
      </div>
    </main>
  );
}