"use client";

import { useState } from "react";
import { useSession } from "@/app/providers";
import { useOrderHistory, useTradeHistory } from "@/lib/use-history";
import { StatusBadge } from "@/components/StatusBadge";

export default function HistoryPage() {
  const { userId } = useSession();
  const [tab, setTab] = useState<"orders" | "trades">("orders");
  const { orders, loading: ordersLoading } = useOrderHistory(userId);
  const { trades, loading: tradesLoading } = useTradeHistory(userId);

  if (!userId) {
    return (
      <main className="min-h-screen bg-gray-950 text-white p-8">
        <p className="text-gray-400">Log in to view your history.</p>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-gray-950 text-white p-8">
      <h1 className="text-2xl font-semibold mb-6">History</h1>

      <div className="flex gap-2 mb-6">
        <button
          onClick={() => setTab("orders")}
          className={`px-4 py-2 rounded ${tab === "orders" ? "bg-indigo-600" : "bg-gray-800"}`}
        >
          Orders
        </button>
        <button
          onClick={() => setTab("trades")}
          className={`px-4 py-2 rounded ${tab === "trades" ? "bg-indigo-600" : "bg-gray-800"}`}
        >
          Trades
        </button>
      </div>

      {tab === "orders" && (
        <>
          {ordersLoading && <p className="text-gray-400">Loading...</p>}
          {!ordersLoading && orders.length === 0 && <p className="text-gray-400">No orders yet.</p>}
          {orders.length > 0 && (
            <table className="w-full text-left">
              <thead className="text-gray-400 border-b border-gray-700">
                <tr>
                  <th className="py-2">Symbol</th>
                  <th className="py-2">Side</th>
                  <th className="py-2">Type</th>
                  <th className="py-2">Price</th>
                  <th className="py-2">Qty</th>
                  <th className="py-2">Filled</th>
                  <th className="py-2">Status</th>
                </tr>
              </thead>
              <tbody>
                {orders.map((o) => (
                  <tr key={o.id} className="border-b border-gray-800">
                    <td className="py-2">{o.symbol}</td>
                    <td className={`py-2 ${o.side === "Buy" ? "text-green-500" : "text-red-500"}`}>{o.side}</td>
                    <td className="py-2">{o.orderType}</td>
                    <td className="py-2">{o.price ? `$${Number(o.price).toFixed(2)}` : "Market"}</td>
                    <td className="py-2">{o.quantity}</td>
                    <td className="py-2">{o.filledQuantity}</td>
                    <td className="py-2">
                      <StatusBadge status={o.status} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}

      {tab === "trades" && (
        <>
          {tradesLoading && <p className="text-gray-400">Loading...</p>}
          {!tradesLoading && trades.length === 0 && <p className="text-gray-400">No trades yet.</p>}
          {trades.length > 0 && (
            <table className="w-full text-left">
              <thead className="text-gray-400 border-b border-gray-700">
                <tr>
                  <th className="py-2">Symbol</th>
                  <th className="py-2">Price</th>
                  <th className="py-2">Qty</th>
                  <th className="py-2">Executed At</th>
                </tr>
              </thead>
              <tbody>
                {trades.map((t) => (
                  <tr key={t.id} className="border-b border-gray-800">
                    <td className="py-2">{t.symbol}</td>
                    <td className="py-2">${Number(t.price).toFixed(2)}</td>
                    <td className="py-2">{t.quantity}</td>
                    <td className="py-2">{new Date(t.executedAt).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      )}
    </main>
  );
}