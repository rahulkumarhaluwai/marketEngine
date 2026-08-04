import { PriceTicker } from "@/components/PriceTicker";

export default function DashboardPage() {
  return (
    <main className="min-h-screen bg-gray-950 text-white p-8">
      <h1 className="text-2xl font-semibold mb-6">Live Market Prices</h1>
      <div className="flex flex-col gap-4">
        <PriceTicker symbol="BTC-USD" label="BTC/USD" />
        <PriceTicker symbol="ETH-USD" label="ETH/USD" />
      </div>
    </main>
  );
}