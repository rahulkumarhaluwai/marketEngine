import { AlertRecord } from "@/lib/use-alerts";

export function AlertList({ alerts }: { alerts: AlertRecord[] }) {
  if (alerts.length === 0) {
    return <p className="text-gray-400">No alerts set.</p>;
  }

  return (
    <table className="w-full text-left text-white">
      <thead className="text-gray-400 border-b border-gray-700">
        <tr>
          <th className="py-2">Symbol</th>
          <th className="py-2">Direction</th>
          <th className="py-2">Target</th>
          <th className="py-2">Status</th>
        </tr>
      </thead>
      <tbody>
        {alerts.map((a) => (
          <tr key={a.id} className="border-b border-gray-800">
            <td className="py-2">{a.symbol}</td>
            <td className="py-2">{a.direction}</td>
            <td className="py-2">${Number(a.targetPrice).toFixed(2)}</td>
            <td className="py-2">
              {a.triggered ? (
                <span className="px-2 py-1 rounded text-xs bg-yellow-900 text-yellow-300">Triggered</span>
              ) : (
                <span className="px-2 py-1 rounded text-xs bg-blue-900 text-blue-300">Active</span>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}