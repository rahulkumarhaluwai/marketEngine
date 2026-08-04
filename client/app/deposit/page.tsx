"use client";

import { useState } from "react";
import { useSession } from "@/app/providers";
import { gqlClient } from "@/lib/graphql-client";
import { DEPOSIT } from "@/lib/queries";

const QUICK_AMOUNTS = ["1000", "5000", "10000", "50000"];

export default function DepositPage() {
  const { userId } = useSession();
  const [amount, setAmount] = useState("");
  const [balance, setBalance] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleDeposit(value: string) {
    if (!userId) return;
    setLoading(true);
    setError(null);
    try {
      const data = await gqlClient.request<{ deposit: { cashBalance: string } }>(DEPOSIT, {
        userId,
        amount: value,
      });
      setBalance(data.deposit.cashBalance);
      setAmount("");
    } catch (err: any) {
      setError(err?.response?.errors?.[0]?.message ?? "Deposit failed");
    } finally {
      setLoading(false);
    }
  }

  if (!userId) {
    return (
      <main className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-white p-8">
        <p className="text-gray-500 dark:text-gray-400">Log in to deposit funds.</p>
      </main>
    );
  }

  return (
    <main className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-white p-8">
      <h1 className="text-2xl font-semibold mb-6">Deposit Virtual Funds</h1>

      <div className="max-w-sm flex flex-col gap-4">
        {balance && (
          <div className="rounded-lg border border-green-300 dark:border-green-700 bg-green-50 dark:bg-green-950 px-4 py-3 text-green-700 dark:text-green-400 text-sm">
            Deposited successfully. New balance: ${Number(balance).toLocaleString()}
          </div>
        )}

        <div className="grid grid-cols-2 gap-2">
          {QUICK_AMOUNTS.map((amt) => (
            <button
              key={amt}
              onClick={() => handleDeposit(amt)}
              disabled={loading}
              className="rounded-lg border border-gray-300 dark:border-gray-700 px-4 py-3 hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-50"
            >
              +${Number(amt).toLocaleString()}
            </button>
          ))}
        </div>

        <div className="flex gap-2">
          <input
            type="text"
            placeholder="Custom amount"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="flex-1 bg-gray-100 dark:bg-gray-800 rounded px-3 py-2 border border-gray-300 dark:border-gray-700"
          />
          <button
            onClick={() => amount && handleDeposit(amount)}
            disabled={loading || !amount}
            className="bg-indigo-600 text-white rounded px-4 py-2 font-medium disabled:opacity-50"
          >
            {loading ? "..." : "Deposit"}
          </button>
        </div>

        {error && <p className="text-red-500 text-sm">{error}</p>}
      </div>
    </main>
  );
}