"use client";

import { useMarketPrice } from "@/lib/use-market-price";

export function PriceTicker({
  symbol,
  label,
  compact = false,
}: {
  symbol: string;
  label: string;
  compact?: boolean;
}) {
  const data = useMarketPrice(symbol);

  const color =
    data?.direction === "up"
      ? "text-green-600 dark:text-green-500"
      : data?.direction === "down"
      ? "text-red-600 dark:text-red-500"
      : "text-gray-500 dark:text-gray-400";

  const priceText = data
    ? `$${Number(data.price).toLocaleString(undefined, { minimumFractionDigits: 2 })}`
    : "—";

  if (compact) {
    return <span className={`font-mono text-sm ${color}`}>{priceText}</span>;
  }

  return (
    <div className="flex items-center justify-between rounded-lg border border-gray-200 dark:border-gray-700 px-4 py-3 w-full max-w-sm">
      <span className="font-medium text-gray-800 dark:text-gray-200">{label}</span>
      <span className={`font-mono text-lg ${color}`}>{priceText}</span>
    </div>
  );
}