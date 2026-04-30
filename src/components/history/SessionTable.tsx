import { useState } from "react";
import { SessionSummaryDto, SessionFilterDto, commands } from "../../bindings";

interface SessionTableProps {
  sessions: SessionSummaryDto[];
  total: number;
  page: number;
  perPage: number;
  onPageChange: (page: number) => void;
  onView: (id: string) => void;
  onCompare: (id: string) => void;
}

export function SessionTable({
  sessions,
  total,
  page,
  perPage,
  onPageChange,
  onView,
  onCompare,
}: SessionTableProps) {
  const [sortKey, setSortKey] = useState<keyof SessionSummaryDto>("created_at");
  const [sortDir, setSortDir] = useState<"asc" | "desc">("desc");

  const sorted = [...sessions].sort((a, b) => {
    const av = a[sortKey];
    const bv = b[sortKey];
    if (av === null || bv === null) return 0;
    if (av < bv) return sortDir === "asc" ? -1 : 1;
    if (av > bv) return sortDir === "asc" ? 1 : -1;
    return 0;
  });

  const toggleSort = (key: keyof SessionSummaryDto) => {
    if (sortKey === key) {
      setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    } else {
      setSortKey(key);
      setSortDir("desc");
    }
  };

  const totalPages = Math.max(1, Math.ceil(total / perPage));

  return (
    <div className="space-y-4">
      <div className="overflow-x-auto">
        <table className="w-full text-sm text-left">
          <thead className="text-xs text-gray-400 uppercase bg-gray-800">
            <tr>
              {[
                { key: "name" as const, label: "Name" },
                { key: "target_space" as const, label: "Target" },
                { key: "tier" as const, label: "Tier" },
                { key: "patch_count" as const, label: "Patches" },
                { key: "gamma" as const, label: "Gamma" },
                { key: "max_de" as const, label: "Max dE" },
                { key: "avg_de" as const, label: "Avg dE" },
              ].map((col) => (
                <th
                  key={col.key}
                  className="px-4 py-3 cursor-pointer select-none"
                  onClick={() => toggleSort(col.key)}
                >
                  {col.label}
                  {sortKey === col.key && (sortDir === "asc" ? " ▲" : " ▼")}
                </th>
              ))}
              <th className="px-4 py-3">Actions</th>
            </tr>
          </thead>
          <tbody>
            {sorted.map((s) => (
              <tr key={s.id} className="border-b border-gray-700 hover:bg-gray-800/50">
                <td className="px-4 py-3 font-medium text-white">
                  <div>{s.name}</div>
                  <div className="text-xs text-gray-500">
                    {new Date(s.created_at).toLocaleDateString()}
                  </div>
                </td>
                <td className="px-4 py-3">{s.target_space}</td>
                <td className="px-4 py-3">
                  <TierBadge tier={s.tier} />
                </td>
                <td className="px-4 py-3">{s.patch_count}</td>
                <td className="px-4 py-3">
                  {s.gamma !== null ? s.gamma.toFixed(2) : "—"}
                </td>
                <td className="px-4 py-3">
                  {s.max_de !== null ? s.max_de.toFixed(2) : "—"}
                </td>
                <td className="px-4 py-3">
                  {s.avg_de !== null ? s.avg_de.toFixed(2) : "—"}
                </td>
                <td className="px-4 py-3 space-x-2">
                  <button
                    className="px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded"
                    onClick={() => onView(s.id)}
                  >
                    View
                  </button>
                  <button
                    className="px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded"
                    onClick={() => onCompare(s.id)}
                  >
                    Compare
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="flex items-center justify-between text-sm text-gray-400">
        <div>
          Page {page + 1} of {totalPages} ({total} total)
        </div>
        <div className="space-x-2">
          <button
            className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-50"
            disabled={page === 0}
            onClick={() => onPageChange(page - 1)}
          >
            Prev
          </button>
          <button
            className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-50"
            disabled={page >= totalPages - 1}
            onClick={() => onPageChange(page + 1)}
          >
            Next
          </button>
        </div>
      </div>
    </div>
  );
}

function TierBadge({ tier }: { tier: string }) {
  const color =
    tier === "Full3D"
      ? "bg-blue-900 text-blue-200"
      : tier === "GrayscalePlus3D"
      ? "bg-purple-900 text-purple-200"
      : "bg-gray-700 text-gray-300";
  return (
    <span className={`px-2 py-0.5 text-xs rounded ${color}`}>
      {tier}
    </span>
  );
}
