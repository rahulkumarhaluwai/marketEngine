"use client";

import { useState } from "react";
import { useSession } from "@/app/providers";
import { useAlerts } from "@/lib/use-alerts";
import { useMarketPrice } from "@/lib/use-market-price";

const SYMBOLS = [
  { value: "BTC_USD", label: "BTC/USD", wsSymbol: "BTC-USD" },
  { value: "ETH_USD", label: "ETH/USD", wsSymbol: "ETH-USD" },
] as const;

export function AlertForm({ onCreated }: { onCreated: () => void }) {
  const { userId } = useSession();
  const [symbolIdx, setSymbolIdx] = useState(0);
  const [targetPrice, setTargetPrice] = useState("");
  const [direction, setDirection] = useState<"ABOVE" | "BELOW">("ABOVE");
  const { createAlert, error } = useAlerts(userId);

  const symbol = SYMBOLS[symbolIdx];
  const liveTick = useMarketPrice(symbol.wsSymbol);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!userId) return;

    const created = await createAlert({
      userId,
      symbol: symbol.value,
      targetPrice,
      direction,
    });

    if (created) {
      setTargetPrice("");
      onCreated();
    }
  }

  return (
    <form onSubmit={handleSubmit} className="max-w-sm flex flex-col gap-4 text-white">
      <div className="flex justify-between items-center">
        <select
          value={symbolIdx}
          onChange={(e) => setSymbolIdx(Number(e.target.value))}
          className="bg-gray-800 rounded px-3 py-2"
        >
          {SYMBOLS.map((s, i) => (
            <option key={s.value} value={i}>
              {s.label}
            </option>
          ))}
        </select>
        <span className="font-mono text-gray-400">
          {liveTick ? `$${Number(liveTick.price).toLocaleString()}` : "—"}
        </span>
      </div>

      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => setDirection("ABOVE")}
          className={`flex-1 py-2 rounded ${direction === "ABOVE" ? "bg-green-600" : "bg-gray-800"}`}
        >
          Above
        </button>
        <button
          type="button"
          onClick={() => setDirection("BELOW")}
          className={`flex-1 py-2 rounded ${direction === "BELOW" ? "bg-red-600" : "bg-gray-800"}`}
        >
          Below
        </button>
      </div>

      <input
        type="text"
        placeholder="Target price"
        value={targetPrice}
        onChange={(e) => setTargetPrice(e.target.value)}
        className="bg-gray-800 rounded px-3 py-2"
        required
      />

      <button type="submit" className="bg-indigo-600 rounded py-2 font-medium">
        Create Alert
      </button>

      {error && <p className="text-red-500 text-sm">{error}</p>}
    </form>
  );
}