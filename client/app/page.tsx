"use client";

import { useState } from "react";
import { PriceTicker } from "@/components/PriceTicker";
import { CandlestickChart } from "@/components/CandlestickChart";

const ASSETS = [
  { symbol: "BTC-USD", label: "BTC/USD", category: "Crypto" },
  { symbol: "ETH-USD", label: "ETH/USD", category: "Crypto" },
  { symbol: "AAPL", label: "Apple", category: "Stocks" },
  { symbol: "TSLA", label: "Tesla", category: "Stocks" },
  { symbol: "GOOGL", label: "Alphabet", category: "Stocks" },
  { symbol: "MSFT", label: "Microsoft", category: "Stocks" },
  { symbol: "AMZN", label: "Amazon", category: "Stocks" },
];

export default function DashboardPage() {
  const [selected, setSelected] = useState("BTC-USD");

  return (
    <main className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-white p-8">
      <h1 className="text-2xl font-semibold mb-6">Live Market Prices</h1>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1 flex flex-col gap-2">
          {ASSETS.map((asset) => (
            <button
              key={asset.symbol}
              onClick={() => setSelected(asset.symbol)}
              className={`text-left rounded-lg border px-4 py-3 transition-colors ${
                selected === asset.symbol
                  ? "border-indigo-500 bg-indigo-50 dark:bg-indigo-950"
                  : "border-gray-200 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-900"
              }`}
            >
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">{asset.label}</div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">{asset.category}</div>
                </div>
                <PriceTicker symbol={asset.symbol} label="" compact />
              </div>
            </button>
          ))}
        </div>

        <div className="lg:col-span-2">
          <CandlestickChart symbol={selected} />
        </div>
      </div>
    </main>
  );
}