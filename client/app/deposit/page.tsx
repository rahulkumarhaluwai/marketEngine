"use client";

import { Suspense, useState } from "react";
import { useSearchParams } from "next/navigation";
import { useSession } from "@/app/providers";
import { gqlClient } from "@/lib/graphql-client";
import { CREATE_CHECKOUT_SESSION } from "@/lib/queries";

const PACKS = [
  { id: "starter", label: "Starter Pack", price: "$1.00", credits: "10,000" },
  { id: "trader", label: "Trader Pack", price: "$5.00", credits: "60,000" },
  { id: "pro", label: "Pro Pack", price: "$10.00", credits: "150,000" },
];

export default function DepositPage() {
  return (
    <Suspense fallback={<main className="min-h-screen bg-white dark:bg-gray-950 p-8" />}>
      <DepositPageContent />
    </Suspense>
  );
}

function DepositPageContent() {
  const { userId } = useSession();
  const searchParams = useSearchParams();
  const [loadingPack, setLoadingPack] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const success = searchParams.get("success") === "true";
  const canceled = searchParams.get("canceled") === "true";

  async function handlePurchase(packId: string) {
    if (!userId) return;
    setLoadingPack(packId);
    setError(null);
    try {
      const data = await gqlClient.request<{ createCheckoutSession: { url: string } }>(CREATE_CHECKOUT_SESSION, {
        userId,
        packId,
      });
      window.location.href = data.createCheckoutSession.url;
    } catch (err: any) {
      setError(err?.response?.errors?.[0]?.message ?? "Could not start checkout");
      setLoadingPack(null);
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
      <h1 className="text-2xl font-semibold mb-2">Buy Virtual Credits</h1>
      <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
        Stripe test mode — no real charges. Use test card 4242 4242 4242 4242, any future date, any CVC.
      </p>

      {success && (
        <div className="mb-6 rounded-lg border border-green-300 dark:border-green-700 bg-green-50 dark:bg-green-950 px-4 py-3 text-green-700 dark:text-green-400 text-sm">
          Payment received. Your balance will update within a few seconds — check{" "}
          <a href="/portfolio" className="underline">Portfolio</a>.
        </div>
      )}
      {canceled && (
        <div className="mb-6 rounded-lg border border-yellow-300 dark:border-yellow-700 bg-yellow-50 dark:bg-yellow-900/30 px-4 py-3 text-yellow-700 dark:text-yellow-300 text-sm">
          Checkout canceled.
        </div>
      )}
      {error && <p className="text-red-500 text-sm mb-4">{error}</p>}

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 max-w-2xl">
        {PACKS.map((pack) => (
          <div key={pack.id} className="rounded-lg border border-gray-300 dark:border-gray-700 p-5 flex flex-col gap-3">
            <div>
              <div className="font-medium">{pack.label}</div>
              <div className="text-2xl font-mono mt-1">{pack.price}</div>
              <div className="text-sm text-gray-500 dark:text-gray-400 mt-1">{pack.credits} virtual credits</div>
            </div>
            <button
              onClick={() => handlePurchase(pack.id)}
              disabled={loadingPack !== null}
              className="bg-indigo-600 text-white rounded py-2 font-medium disabled:opacity-50"
            >
              {loadingPack === pack.id ? "Redirecting..." : "Buy"}
            </button>
          </div>
        ))}
      </div>
    </main>
  );
}