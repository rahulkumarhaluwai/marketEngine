"use client";

import { useState } from "react";
import { useMarketPrice } from "@/lib/use-market-price";
import { usePlaceOrder } from "@/lib/use-place-order";
import { useSession } from "@/app/providers";

const SYMBOLS = [
  { value: "BTC_USD", label: "BTC/USD", wsSymbol: "BTC-USD" },
  { value: "ETH_USD", label: "ETH/USD", wsSymbol: "ETH-USD" },
  { value: "AAPL", label: "Apple (AAPL)", wsSymbol: "AAPL" },
  { value: "TSLA", label: "Tesla (TSLA)", wsSymbol: "TSLA" },
  { value: "GOOGL", label: "Alphabet (GOOGL)", wsSymbol: "GOOGL" },
  { value: "MSFT", label: "Microsoft (MSFT)", wsSymbol: "MSFT" },
  { value: "AMZN", label: "Amazon (AMZN)", wsSymbol: "AMZN" },
] as const;

export function OrderForm() {
  const { userId } = useSession();
  const [symbolIdx, setSymbolIdx] = useState(0);
  const [side, setSide] = useState<"BUY" | "SELL">("BUY");
  const [orderType, setOrderType] = useState<"MARKET" | "LIMIT">("MARKET");
  const [quantity, setQuantity] = useState("");
  const [price, setPrice] = useState("");

  const { placeOrder, loading, error, result } = usePlaceOrder();
  const symbol = SYMBOLS[symbolIdx];
  const liveTick = useMarketPrice(symbol.wsSymbol);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!userId) return;

    await placeOrder({
      userId,
      symbol: symbol.value,
      side,
      orderType,
      quantity,
      price: orderType === "LIMIT" ? price : undefined,
    });
  }

  if (!userId) {
    return <p className="text-gray-500 dark:text-gray-400">Log in to place orders.</p>;
  }

  return (
    <form onSubmit={handleSubmit} className="max-w-sm flex flex-col gap-4 text-gray-900 dark:text-white">
      <div className="flex justify-between items-center">
        <select
          value={symbolIdx}
          onChange={(e) => setSymbolIdx(Number(e.target.value))}
          className="bg-gray-100 dark:bg-gray-800 rounded px-3 py-2 border border-gray-300 dark:border-gray-700"
        >
          {SYMBOLS.map((s, i) => (
            <option key={s.value} value={i}>
              {s.label}
            </option>
          ))}
        </select>
        <span className="font-mono text-gray-500 dark:text-gray-400">
          {liveTick ? `$${Number(liveTick.price).toLocaleString()}` : "—"}
        </span>
      </div>

      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => setSide("BUY")}
          className={`flex-1 py-2 rounded ${side === "BUY" ? "bg-green-600 text-white" : "bg-gray-100 dark:bg-gray-800"}`}
        >
          Buy
        </button>
        <button
          type="button"
          onClick={() => setSide("SELL")}
          className={`flex-1 py-2 rounded ${side === "SELL" ? "bg-red-600 text-white" : "bg-gray-100 dark:bg-gray-800"}`}
        >
          Sell
        </button>
      </div>

      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => setOrderType("MARKET")}
          className={`flex-1 py-2 rounded ${orderType === "MARKET" ? "bg-blue-600 text-white" : "bg-gray-100 dark:bg-gray-800"}`}
        >
          Market
        </button>
        <button
          type="button"
          onClick={() => setOrderType("LIMIT")}
          className={`flex-1 py-2 rounded ${orderType === "LIMIT" ? "bg-blue-600 text-white" : "bg-gray-100 dark:bg-gray-800"}`}
        >
          Limit
        </button>
      </div>

      <input
        type="text"
        placeholder="Quantity"
        value={quantity}
        onChange={(e) => setQuantity(e.target.value)}
        className="bg-gray-100 dark:bg-gray-800 rounded px-3 py-2 border border-gray-300 dark:border-gray-700"
        required
      />

      {orderType === "LIMIT" && (
        <input
          type="text"
          placeholder="Limit price"
          value={price}
          onChange={(e) => setPrice(e.target.value)}
          className="bg-gray-100 dark:bg-gray-800 rounded px-3 py-2 border border-gray-300 dark:border-gray-700"
          required
        />
      )}

      <button type="submit" disabled={loading} className="bg-indigo-600 text-white rounded py-2 font-medium disabled:opacity-50">
        {loading ? "Placing..." : "Place Order"}
      </button>

      {error && <p className="text-red-500 text-sm">{error}</p>}
      {result && (
        <p className="text-green-600 dark:text-green-500 text-sm">
          Order {result.status}: {result.filledQuantity}/{result.quantity} filled
        </p>
      )}
    </form>
  );
}