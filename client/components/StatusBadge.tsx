const STATUS_COLORS: Record<string, string> = {
  Open: "bg-blue-900 text-blue-300",
  PartiallyFilled: "bg-yellow-900 text-yellow-300",
  Filled: "bg-green-900 text-green-300",
  Cancelled: "bg-gray-800 text-gray-400",
};

export function StatusBadge({ status }: { status: string }) {
  const color = STATUS_COLORS[status] ?? "bg-gray-800 text-gray-400";
  return <span className={`px-2 py-1 rounded text-xs font-medium ${color}`}>{status}</span>;
}