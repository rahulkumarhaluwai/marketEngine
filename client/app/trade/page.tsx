import { OrderForm } from "@/components/OrderForm";

export default function TradePage() {
  return (
    <main className="min-h-screen bg-gray-950 text-white p-8">
      <h1 className="text-2xl font-semibold mb-6">Place Order</h1>
      <OrderForm />
    </main>
  );
}