import { DeBarChart } from "./DeBarChart";
import { LiveGammaChart } from "./LiveGammaChart";
import { PatchDataTable } from "./PatchDataTable";
import type { PatchReading, AnalysisResult } from "./types";

export function AnalysisStep({
  readings,
  analysis,
  onApply,
  onRemeasure,
}: {
  readings: PatchReading[];
  analysis: AnalysisResult;
  onApply: () => void;
  onRemeasure: () => void;
}) {
  const dePoints = readings.map((r) => ({ level: (r.patch_index / readings.length) * 100, de: r.de2000 }));
  const gammaPoints = readings.map((r) => ({ level: (r.patch_index / readings.length) * 100, y: r.yxy[0] }));

  return (
    <div className="space-y-6">
      {/* Summary cards */}
      <div className="grid grid-cols-4 gap-4">
        <SummaryCard label="Estimated Gamma" value={analysis.gamma.toFixed(2)} />
        <SummaryCard label="Max dE2000" value={analysis.max_de.toFixed(2)} color={analysis.max_de < 1 ? "green" : analysis.max_de < 3 ? "yellow" : "red"} />
        <SummaryCard label="Avg dE2000" value={analysis.avg_de.toFixed(2)} color={analysis.avg_de < 1 ? "green" : analysis.avg_de < 3 ? "yellow" : "red"} />
        <SummaryCard label="White Balance" value={`R${analysis.white_balance_errors[0].toFixed(2)} G${analysis.white_balance_errors[1].toFixed(2)} B${analysis.white_balance_errors[2].toFixed(2)}`} />
      </div>

      {/* Charts */}
      <div className="space-y-4">
        <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
          <div className="text-xs text-gray-500 uppercase mb-2">dE2000 per Patch</div>
          <DeBarChart points={dePoints} />
        </div>
        <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
          <div className="text-xs text-gray-500 uppercase mb-2">Gamma Curve</div>
          <LiveGammaChart targetGamma={2.4} measuredPoints={gammaPoints} />
        </div>
      </div>

      {/* Table */}
      <PatchDataTable readings={readings} />

      {/* Actions */}
      <div className="flex justify-between">
        <button
          onClick={onRemeasure}
          className="px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition"
        >
          Re-measure
        </button>
        <button
          onClick={onApply}
          className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition"
        >
          Apply Corrections
        </button>
      </div>
    </div>
  );
}

function SummaryCard({
  label,
  value,
  color = "white",
}: {
  label: string;
  value: string;
  color?: "white" | "green" | "yellow" | "red";
}) {
  const colorClass = {
    white: "text-white",
    green: "text-green-500",
    yellow: "text-yellow-500",
    red: "text-red-500",
  }[color];

  return (
    <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
      <div className="text-xs text-gray-500 uppercase">{label}</div>
      <div className={`text-xl font-semibold ${colorClass}`}>{value}</div>
    </div>
  );
}
