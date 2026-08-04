"use client";

import { useMarketPrice } from "@/lib/use-market-price";

export function PriceTicker({ symbol, label }: { symbol: string; label: string }) {
  const data = useMarketPrice(symbol);

  const color =
    data?.direction === "up" ? "text-green-500" : data?.direction === "down" ? "text-red-500" : "text-gray-400";

  return (
    <div className="flex items-center justify-between rounded-lg border border-gray-700 px-4 py-3 w-full max-w-sm">
      <span className="font-medium text-gray-200">{label}</span>
      <span className={`font-mono text-lg ${color}`}>
        {data ? `$${Number(data.price).toLocaleString(undefined, { minimumFractionDigits: 2 })}` : "—"}
      </span>
    </div>
  );
}