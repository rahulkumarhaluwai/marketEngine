"use client";

import { useMarketPrice } from "@/lib/use-market-price";
import { Position } from "@/lib/use-portfolio";

function PositionRow({ position }: { position: Position }) {
  const liveTick = useMarketPrice(position.symbol);

  const quantity = parseFloat(position.quantity);
  const avgCost = parseFloat(position.avgCost);
  const livePrice = liveTick ? parseFloat(liveTick.price) : parseFloat(position.marketValue) / quantity;
  const liveMarketValue = quantity * livePrice;
  const livePnl = liveMarketValue - avgCost * quantity;

  const pnlColor = livePnl >= 0 ? "text-green-600 dark:text-green-500" : "text-red-600 dark:text-red-500";

  return (
    <tr className="border-b border-gray-200 dark:border-gray-800">
      <td className="py-2">{position.symbol}</td>
      <td className="py-2">{quantity.toFixed(6)}</td>
      <td className="py-2">${avgCost.toFixed(2)}</td>
      <td className="py-2">${liveMarketValue.toFixed(2)}</td>
      <td className={`py-2 ${pnlColor}`}>
        {livePnl >= 0 ? "+" : ""}
        {livePnl.toFixed(2)}
      </td>
    </tr>
  );
}

export function PortfolioTable({ positions }: { positions: Position[] }) {
  if (positions.length === 0) {
    return <p className="text-gray-500 dark:text-gray-400">No open positions.</p>;
  }

  return (
    <table className="w-full text-left text-gray-900 dark:text-white">
      <thead className="text-gray-500 dark:text-gray-400 border-b border-gray-300 dark:border-gray-700">
        <tr>
          <th className="py-2">Symbol</th>
          <th className="py-2">Qty</th>
          <th className="py-2">Avg Cost</th>
          <th className="py-2">Market Value</th>
          <th className="py-2">Unrealized P&L</th>
        </tr>
      </thead>
      <tbody>
        {positions.map((p) => (
          <PositionRow key={p.symbol} position={p} />
        ))}
      </tbody>
    </table>
  );
}