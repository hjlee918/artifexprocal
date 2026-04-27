import { CIEDiagram } from "../visualizations/CIEDiagram";
import { GrayscaleTracker, type GrayscalePoint } from "../visualizations/GrayscaleTracker";
import { PatchDataTable } from "./PatchDataTable";
import { getTargetGamut } from "../../lib/colorMath";
import type { PatchReading, AnalysisResult } from "./types";

export function AnalysisStep({
  readings,
  analysis,
  targetSpace,
  onApply,
  onRemeasure,
}: {
  readings: PatchReading[];
  analysis: AnalysisResult;
  targetSpace?: string;
  onApply: () => void;
  onRemeasure: () => void;
}) {
  const gammaPoints: GrayscalePoint[] = readings.map((r) => ({
    level: (r.patch_index / readings.length) * 100,
    r: r.rgb[0],
    g: r.rgb[1],
    b: r.rgb[2],
    y: r.yxy[0],
    de: r.de2000,
    x: r.yxy[1],
    y_chromaticity: r.yxy[2],
  }));

  // Minimal mock spectral locus for rendering (will be fetched from backend in future)
  const locus: [number, number][] = [
    [0.174, 0.005], [0.173, 0.005], [0.171, 0.005], [0.166, 0.009],
    [0.161, 0.014], [0.151, 0.023], [0.144, 0.03], [0.128, 0.055],
    [0.112, 0.103], [0.104, 0.136], [0.098, 0.173], [0.092, 0.212],
    [0.088, 0.251], [0.081, 0.322], [0.076, 0.394], [0.072, 0.438],
    [0.071, 0.442], [0.07, 0.439], [0.069, 0.435], [0.066, 0.409],
    [0.063, 0.379], [0.059, 0.342], [0.055, 0.301], [0.051, 0.258],
    [0.046, 0.216], [0.042, 0.177], [0.039, 0.142], [0.035, 0.111],
    [0.033, 0.084], [0.03, 0.061], [0.029, 0.051], [0.028, 0.042],
    [0.028, 0.034], [0.027, 0.027], [0.027, 0.021], [0.027, 0.016],
    [0.027, 0.012], [0.026, 0.009], [0.026, 0.006], [0.026, 0.004],
    [0.026, 0.003], [0.026, 0.002], [0.026, 0.001],
  ];

  const targetGamut = getTargetGamut(targetSpace ?? "Rec.709");

  const measuredGamut = {
    red: [0.64, 0.33] as [number, number],
    green: [0.3, 0.6] as [number, number],
    blue: [0.15, 0.06] as [number, number],
    white: [0.3127, 0.329] as [number, number],
  };

  return (
    <div className="space-y-6">
      {/* Summary cards */}
      <div className="grid grid-cols-4 gap-4">
        <SummaryCard label="Estimated Gamma" value={analysis.gamma.toFixed(2)} />
        <SummaryCard
          label="Max dE2000"
          value={analysis.max_de.toFixed(2)}
          color={analysis.max_de < 1 ? "green" : analysis.max_de < 3 ? "yellow" : "red"}
        />
        <SummaryCard
          label="Avg dE2000"
          value={analysis.avg_de.toFixed(2)}
          color={analysis.avg_de < 1 ? "green" : analysis.avg_de < 3 ? "yellow" : "red"}
        />
        <SummaryCard
          label="White Balance"
          value={`R${analysis.white_balance_errors[0].toFixed(2)} G${analysis.white_balance_errors[1].toFixed(2)} B${analysis.white_balance_errors[2].toFixed(2)}`}
        />
      </div>

      {/* Grayscale Tracker */}
      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase mb-2">Grayscale Tracker</div>
        <GrayscaleTracker targetGamma={2.4} points={gammaPoints} />
      </div>

      {/* CIE Diagram */}
      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase mb-2">CIE 1931 xy Chromaticity</div>
        <CIEDiagram locus={locus} targetGamut={targetGamut} measuredGamut={measuredGamut} />
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
