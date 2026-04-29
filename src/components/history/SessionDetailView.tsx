import { useState } from "react";
import { SessionDetailDto } from "../../bindings";

interface SessionDetailViewProps {
  detail: SessionDetailDto;
  onBack: () => void;
  onExport: (format: string) => void;
  onCompare: () => void;
}

export function SessionDetailView({
  detail,
  onBack,
  onExport,
  onCompare,
}: SessionDetailViewProps) {
  const [tab, setTab] = useState<"summary" | "readings">("summary");

  const s = detail.summary;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <button
          className="text-sm text-gray-400 hover:text-white"
          onClick={onBack}
        >
          ← Back to History
        </button>
        <div className="space-x-2">
          <button
            className="px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 rounded"
            onClick={() => onExport("csv")}
          >
            Export CSV
          </button>
          <button
            className="px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 rounded"
            onClick={() => onExport("json")}
          >
            Export JSON
          </button>
          <button
            className="px-3 py-1.5 text-sm bg-blue-700 hover:bg-blue-600 rounded"
            onClick={onCompare}
          >
            Compare
          </button>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <SummaryCard label="Name" value={s.name} />
        <SummaryCard label="State" value={s.state} />
        <SummaryCard label="Target" value={s.target_space} />
        <SummaryCard label="Tier" value={s.tier} />
        <SummaryCard label="Patches" value={String(s.patch_count)} />
        <SummaryCard
          label="Gamma"
          value={s.gamma !== null ? s.gamma.toFixed(2) : "—"}
        />
        <SummaryCard
          label="Max dE"
          value={s.max_de !== null ? s.max_de.toFixed(2) : "—"}
        />
        <SummaryCard
          label="Avg dE"
          value={s.avg_de !== null ? s.avg_de.toFixed(2) : "—"}
        />
      </div>

      {/* Tabs */}
      <div className="flex space-x-4 border-b border-gray-700">
        {(["summary", "readings"] as const).map((t) => (
          <button
            key={t}
            className={`pb-2 text-sm capitalize ${
              tab === t
                ? "text-white border-b-2 border-blue-500"
                : "text-gray-400 hover:text-gray-200"
            }`}
            onClick={() => setTab(t)}
          >
            {t}
          </button>
        ))}
      </div>

      {tab === "readings" && (
        <div className="overflow-x-auto">
          <table className="w-full text-sm text-left">
            <thead className="text-xs text-gray-400 uppercase bg-gray-800">
              <tr>
                <th className="px-4 py-2">Patch</th>
                <th className="px-4 py-2">Target RGB</th>
                <th className="px-4 py-2">Measured XYZ</th>
                <th className="px-4 py-2">Type</th>
              </tr>
            </thead>
            <tbody>
              {detail.readings.map((r) => (
                <tr
                  key={`${r.patch_index}-${r.reading_index}`}
                  className="border-b border-gray-700"
                >
                  <td className="px-4 py-2">{r.patch_index}</td>
                  <td className="px-4 py-2">
                    [{r.target_rgb[0].toFixed(2)}, {r.target_rgb[1].toFixed(2)},{" "}
                    {r.target_rgb[2].toFixed(2)}]
                  </td>
                  <td className="px-4 py-2">
                    [{r.measured_xyz[0].toFixed(2)}, {r.measured_xyz[1].toFixed(2)},{" "}
                    {r.measured_xyz[2].toFixed(2)}]
                  </td>
                  <td className="px-4 py-2">{r.measurement_type}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function SummaryCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-3">
      <div className="text-xs text-gray-500 uppercase">{label}</div>
      <div className="text-lg font-semibold text-white">{value}</div>
    </div>
  );
}
