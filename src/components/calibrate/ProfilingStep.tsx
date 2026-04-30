import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { startProfiling } from "../../bindings";
import { EVENT_PROFILING_PROGRESS, EVENT_PROFILING_COMPLETE } from "../../bindings";

interface ProfilingProgress {
  session_id: string;
  current_patch: number;
  total_patches: number;
  patch_name: string;
  reference_xyz: [number, number, number];
  meter_xyz: [number, number, number];
  delta_e: number;
}

interface ProfilingCompletePayload {
  session_id: string;
  correction_matrix: number[][];
  accuracy_estimate: number;
}

export function ProfilingStep({
  meterId,
  referenceMeterId,
  displayId,
  onComplete,
  onSkip,
}: {
  meterId: string;
  referenceMeterId: string;
  displayId: string;
  onComplete: (matrix: number[][], accuracy: number) => void;
  onSkip: () => void;
}) {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [currentPatch, setCurrentPatch] = useState(0);
  const [totalPatches] = useState(20);
  const [patchName, setPatchName] = useState("Starting...");
  const [results, setResults] = useState<ProfilingProgress[]>([]);
  const [correctionMatrix, setCorrectionMatrix] = useState<number[][] | null>(null);
  const [accuracyEstimate, setAccuracyEstimate] = useState<number | null>(null);
  const [isComplete, setIsComplete] = useState(false);

  useEffect(() => {
    startProfiling(meterId, referenceMeterId, displayId, {
      patch_set: "full",
      patch_scale: "legal",
    }).then((sid) => {
      setSessionId(sid);
    });
  }, [meterId, referenceMeterId, displayId]);

  useEffect(() => {
    if (!sessionId) return;
    let cancelled = false;
    const unsubPromise = listen<ProfilingProgress>(EVENT_PROFILING_PROGRESS, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      const p = event.payload;
      setCurrentPatch(p.current_patch);
      setPatchName(p.patch_name);
      setResults((prev) => {
        const filtered = prev.filter((r) => r.current_patch !== p.current_patch);
        return [...filtered, p];
      });
    });
    const completeUnsubPromise = listen<ProfilingCompletePayload>(EVENT_PROFILING_COMPLETE, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      setCorrectionMatrix(event.payload.correction_matrix);
      setAccuracyEstimate(event.payload.accuracy_estimate);
      setIsComplete(true);
    });
    return () => {
      cancelled = true;
      unsubPromise.then((u) => u());
      completeUnsubPromise.then((u) => u());
    };
  }, [sessionId]);

  const progress = (currentPatch / totalPatches) * 100;
  const avgDe = results.length > 0 ? results.reduce((a, b) => a + b.delta_e, 0) / results.length : 0;

  return (
    <div className="space-y-4">
      <div className="text-sm text-gray-400">
        Profiling {meterId} against {referenceMeterId} — Patch {currentPatch} of {totalPatches} ({patchName})
      </div>
      <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
        <div className="h-full bg-primary rounded-full transition-all" style={{ width: `${progress}%` }} />
      </div>

      {/* Results table */}
      <div className="border border-gray-800 rounded-lg overflow-hidden max-h-40 overflow-y-auto">
        <table className="w-full text-xs">
          <thead className="bg-gray-800 text-gray-400 sticky top-0">
            <tr>
              <th className="text-left px-3 py-2">Patch</th>
              <th className="text-right px-3 py-2">Ref XYZ</th>
              <th className="text-right px-3 py-2">Meter XYZ</th>
              <th className="text-right px-3 py-2">dE</th>
            </tr>
          </thead>
          <tbody>
            {results.map((r) => (
              <tr key={r.current_patch} className="border-t border-gray-800">
                <td className="px-3 py-1.5">{r.patch_name}</td>
                <td className="px-3 py-1.5 text-right text-gray-400">
                  {r.reference_xyz.map((v) => v.toFixed(1)).join(", ")}
                </td>
                <td className="px-3 py-1.5 text-right text-gray-400">
                  {r.meter_xyz.map((v) => v.toFixed(1)).join(", ")}
                </td>
                <td className="px-3 py-1.5 text-right">{r.delta_e.toFixed(2)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Matrix preview */}
      {isComplete && correctionMatrix && (
        <div className="bg-gray-800 border border-gray-800 rounded-lg p-4">
          <div className="text-sm font-medium mb-2">Correction Matrix</div>
          <div className="text-xs text-gray-400 mb-2">
            Average dE: {accuracyEstimate?.toFixed(2) ?? avgDe.toFixed(2)}
          </div>
          <div className="grid grid-cols-3 gap-2 text-sm font-mono">
            {correctionMatrix.flat().map((v, i) => (
              <div key={i} className="bg-gray-900 border border-gray-800 rounded px-2 py-1 text-center">{v.toFixed(4)}</div>
            ))}
          </div>
          <div className="flex gap-3 mt-4">
            <button
              onClick={() => onComplete(correctionMatrix, accuracyEstimate ?? avgDe)}
              className="px-4 py-2 rounded-lg bg-primary text-white text-sm hover:bg-sky-400 transition"
            >
              Accept &amp; Save
            </button>
            <button onClick={onSkip} className="px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition">
              Skip
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
