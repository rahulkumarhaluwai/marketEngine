"use client";

import { useEffect, useState } from "react";
import { gqlClient } from "@/lib/graphql-client";
import { GET_LEADERBOARD } from "@/lib/queries";

type Entry = { rank: number; username: string; equity: string };

export default function LeaderboardPage() {
  const [entries, setEntries] = useState<Entry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    gqlClient
      .request<{ leaderboard: Entry[] }>(GET_LEADERBOARD, { limit: 20 })
      .then((data) => setEntries(data.leaderboard))
      .finally(() => setLoading(false));
  }, []);

  return (
    <main className="min-h-screen bg-white dark:bg-gray-950 text-gray-900 dark:text-white p-8">
      <h1 className="text-2xl font-semibold mb-6">Leaderboard</h1>
      <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">Ranked by total account equity (cash + holdings value), updated every ~10s.</p>

      {loading && <p className="text-gray-500 dark:text-gray-400">Loading...</p>}
      {!loading && entries.length === 0 && <p className="text-gray-500 dark:text-gray-400">No accounts yet.</p>}

      {entries.length > 0 && (
        <table className="w-full text-left max-w-lg">
          <thead className="text-gray-500 dark:text-gray-400 border-b border-gray-300 dark:border-gray-700">
            <tr>
              <th className="py-2">Rank</th>
              <th className="py-2">User</th>
              <th className="py-2">Equity</th>
            </tr>
          </thead>
          <tbody>
            {entries.map((e) => (
              <tr key={e.rank} className="border-b border-gray-200 dark:border-gray-800">
                <td className="py-2 font-mono">#{e.rank}</td>
                <td className="py-2">
                  <div className="flex items-center gap-2">
                    {e.username}
                  </div>
                </td>
                <td className="py-2 font-mono">${Number(e.equity).toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </main>
  );
}