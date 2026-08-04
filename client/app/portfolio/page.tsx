"use client";

import { useSession } from "@/app/providers";
import { usePortfolio } from "@/lib/use-portfolio";
import { PortfolioTable } from "@/components/PortfolioTable";

export default function PortfolioPage() {
  const { userId } = useSession();
  const { portfolio, loading, error, refresh } = usePortfolio(userId);

  if (!userId) {
    return (
      <main className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-white p-8">
        <p className="text-gray-500 dark:text-gray-400">Log in to view your portfolio.</p>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-white p-8">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-semibold">Portfolio</h1>
        <button onClick={refresh} className="text-sm text-indigo-600 dark:text-indigo-400 hover:underline">
          Refresh
        </button>
      </div>

      {loading && <p className="text-gray-500 dark:text-gray-400">Loading...</p>}
      {error && <p className="text-red-500">{error}</p>}

      {portfolio && (
        <>
          <div className="grid grid-cols-3 gap-4 mb-8 max-w-2xl">
            <div className="rounded-lg border border-gray-300 dark:border-gray-700 p-4">
              <p className="text-gray-500 dark:text-gray-400 text-sm">Cash Balance</p>
              <p className="text-xl font-mono">${Number(portfolio.cashBalance).toFixed(2)}</p>
            </div>
            <div className="rounded-lg border border-gray-300 dark:border-gray-700 p-4">
              <p className="text-gray-500 dark:text-gray-400 text-sm">Market Value</p>
              <p className="text-xl font-mono">${Number(portfolio.totalMarketValue).toFixed(2)}</p>
            </div>
            <div className="rounded-lg border border-gray-300 dark:border-gray-700 p-4">
              <p className="text-gray-500 dark:text-gray-400 text-sm">Unrealized P&L</p>
              <p
                className={`text-xl font-mono ${
                  Number(portfolio.totalUnrealizedPnl) >= 0 ? "text-green-600 dark:text-green-500" : "text-red-600 dark:text-red-500"
                }`}
              >
                {Number(portfolio.totalUnrealizedPnl) >= 0 ? "+" : ""}
                {Number(portfolio.totalUnrealizedPnl).toFixed(2)}
              </p>
            </div>
          </div>

          <PortfolioTable positions={portfolio.positions} />
        </>
      )}
    </main>
  );
}