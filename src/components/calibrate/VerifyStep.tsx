import { DeBarChart } from "./DeBarChart";
import type { VerificationResult } from "./types";
import { CheckCircle, AlertTriangle, XCircle } from "lucide-react";
import { useNavigate } from "react-router-dom";

export function VerifyStep({
  result,
  onSave,
}: {
  result: VerificationResult;
  onSave: () => void;
}) {
  const navigate = useNavigate();
  const maxPostDe = Math.max(...result.post_de);
  const avgPreDe = result.pre_de.reduce((a, b) => a + b, 0) / result.pre_de.length;
  const avgPostDe = result.post_de.reduce((a, b) => a + b, 0) / result.post_de.length;
  const improvement = avgPreDe > 0 ? ((avgPreDe - avgPostDe) / avgPreDe) * 100 : 0;

  const verdict =
    maxPostDe < 1
      ? { icon: <CheckCircle size={32} className="text-green-500" />, text: "Pass", color: "text-green-500" }
      : maxPostDe < 3
        ? { icon: <AlertTriangle size={32} className="text-yellow-500" />, text: "Marginal", color: "text-yellow-500" }
        : { icon: <XCircle size={32} className="text-red-500" />, text: "Fail", color: "text-red-500" };

  const dePoints = result.pre_de.map((pre, i) => ({
    level: (i / result.pre_de.length) * 100,
    de: pre,
  }));

  return (
    <div className="space-y-6">
      {/* Verdict */}
      <div className="flex flex-col items-center py-6">
        {verdict.icon}
        <div className={`text-xl font-semibold mt-2 ${verdict.color}`}>{verdict.text}</div>
        <div className="text-sm text-gray-400">Max post-calibration dE2000: {maxPostDe.toFixed(2)}</div>
      </div>

      {/* Summary */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-gray-800 border border-gray-800 rounded-lg p-3 text-center">
          <div className="text-xs text-gray-500">Pre Avg dE</div>
          <div className="text-xl font-semibold text-white">{avgPreDe.toFixed(2)}</div>
        </div>
        <div className="bg-gray-800 border border-gray-800 rounded-lg p-3 text-center">
          <div className="text-xs text-gray-500">Post Avg dE</div>
          <div className="text-xl font-semibold text-white">{avgPostDe.toFixed(2)}</div>
        </div>
        <div className="bg-gray-800 border border-gray-800 rounded-lg p-3 text-center">
          <div className="text-xs text-gray-500">Improvement</div>
          <div className="text-xl font-semibold text-green-500">{improvement.toFixed(1)}%</div>
        </div>
      </div>

      {/* Comparison chart */}
      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase mb-2">Pre vs Post dE2000</div>
        <DeBarChart points={dePoints} />
      </div>

      {/* Actions */}
      <div className="flex justify-between">
        <button
          onClick={() => navigate("/")}
          className="px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition"
        >
          Back to Dashboard
        </button>
        <div className="flex gap-3">
          <button
            onClick={onSave}
            className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition"
          >
            Save Session
          </button>
        </div>
      </div>
    </div>
  );
}
