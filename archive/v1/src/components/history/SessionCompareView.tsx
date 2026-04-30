import { SessionDetailDto, ComputedResultsDto } from "../../bindings";

interface SessionCompareViewProps {
  sessionA: SessionDetailDto;
  sessionB: SessionDetailDto;
  onBack: () => void;
}

interface MetricRow {
  label: string;
  key: keyof ComputedResultsDto;
  format: (v: number | null) => string;
  lowerIsBetter: boolean;
}

const METRICS: MetricRow[] = [
  {
    label: "Gamma",
    key: "gamma",
    format: (v) => (v !== null ? v.toFixed(2) : "—"),
    lowerIsBetter: false,
  },
  {
    label: "Max dE2000",
    key: "max_de",
    format: (v) => (v !== null ? v.toFixed(2) : "—"),
    lowerIsBetter: true,
  },
  {
    label: "Avg dE2000",
    key: "avg_de",
    format: (v) => (v !== null ? v.toFixed(2) : "—"),
    lowerIsBetter: true,
  },
];

export function SessionCompareView({
  sessionA,
  sessionB,
  onBack,
}: SessionCompareViewProps) {
  const sa = sessionA.summary;
  const sb = sessionB.summary;
  const ra = sessionA.results;
  const rb = sessionB.results;

  return (
    <div className="space-y-6">
      <button
        className="text-sm text-gray-400 hover:text-white"
        onClick={onBack}
      >
        ← Back
      </button>

      <div className="grid grid-cols-3 gap-4 text-center">
        <div>
          <div className="text-xs text-gray-500 uppercase">Session A</div>
          <div className="text-lg font-semibold text-white">{sa.name}</div>
          <div className="text-xs text-gray-400">
            {new Date(sa.created_at).toLocaleDateString()}
          </div>
        </div>
        <div className="flex items-center justify-center">
          <span className="text-gray-500">vs</span>
        </div>
        <div>
          <div className="text-xs text-gray-500 uppercase">Session B</div>
          <div className="text-lg font-semibold text-white">{sb.name}</div>
          <div className="text-xs text-gray-400">
            {new Date(sb.created_at).toLocaleDateString()}
          </div>
        </div>
      </div>

      <div className="bg-gray-800 border border-gray-700 rounded-lg overflow-hidden">
        <table className="w-full text-sm text-left">
          <thead className="text-xs text-gray-400 uppercase bg-gray-900">
            <tr>
              <th className="px-4 py-3">Metric</th>
              <th className="px-4 py-3 text-right">Before (A)</th>
              <th className="px-4 py-3 text-right">After (B)</th>
              <th className="px-4 py-3 text-right">Delta</th>
            </tr>
          </thead>
          <tbody>
            {METRICS.map((m) => {
              const va = ra?.[m.key] as number | null;
              const vb = rb?.[m.key] as number | null;
              const delta =
                va !== null && vb !== null ? vb - va : null;
              const improved =
                delta !== null
                  ? m.lowerIsBetter
                    ? delta < 0
                    : Math.abs(delta) < 0.05
                  : null;

              return (
                <tr key={m.label} className="border-b border-gray-700">
                  <td className="px-4 py-3">{m.label}</td>
                  <td className="px-4 py-3 text-right">{m.format(va)}</td>
                  <td className="px-4 py-3 text-right">{m.format(vb)}</td>
                  <td
                    className={`px-4 py-3 text-right font-medium ${
                      improved === true
                        ? "text-green-400"
                        : improved === false
                        ? "text-red-400"
                        : "text-gray-400"
                    }`}
                  >
                    {delta !== null ? `${delta >= 0 ? "+" : ""}${delta.toFixed(2)}` : "—"}
                    {improved === true && " ✓"}
                    {improved === false && " ✗"}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
