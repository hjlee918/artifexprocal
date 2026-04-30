import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  startManualCalibration,
  measureManualPatch,
  nextManualPatch,
  prevManualPatch,
  skipManualPatch,
  finishManualCalibration,
  abortManualCalibration,
  getManualCalibrationState,
  EVENT_MANUAL_PATCH_DISPLAYED,
  EVENT_MANUAL_PATCH_MEASURED,
  EVENT_MANUAL_PATCH_SKIPPED,
  EVENT_MANUAL_CALIBRATION_COMPLETE,
} from "../../bindings";
import type { ManualPatchDto } from "../../bindings";

export function ManualCalibrationStep({
  config,
  onComplete,
  onAbort,
}: {
  config: import("../../bindings").ManualConfigDto;
  onComplete: () => void;
  onAbort: () => void;
}) {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [currentPatch, setCurrentPatch] = useState(0);
  const [totalPatches, setTotalPatches] = useState(0);
  const [patchName, setPatchName] = useState("");
  const [targetRgb, setTargetRgb] = useState<[number, number, number] | null>(null);
  const [measuredXyz, setMeasuredXyz] = useState<[number, number, number] | null>(null);
  const [deltaE, setDeltaE] = useState<number | null>(null);
  const [isMeasuring, setIsMeasuring] = useState(false);
  const [isComplete, setIsComplete] = useState(false);
  const [patches, setPatches] = useState<ManualPatchDto[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    startManualCalibration(config)
      .then((sid) => {
        if (cancelled) return;
        setSessionId(sid);
        return getManualCalibrationState();
      })
      .then((state) => {
        if (cancelled || !state) return;
        setCurrentPatch(state.current_patch);
        setTotalPatches(state.total_patches);
        setPatches(state.patches);
        const cp = state.patches[state.current_patch];
        if (cp) {
          setPatchName(cp.patch_type);
          setTargetRgb(cp.target_rgb);
          setMeasuredXyz(cp.measured_xyz ?? null);
          setDeltaE(cp.delta_e ?? null);
        }
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });

    return () => {
      cancelled = true;
    };
  }, [config]);

  useEffect(() => {
    if (!sessionId) return;
    let cancelled = false;

    const unsubDisplayed = listen<{
      session_id: string;
      patch_index: number;
      patch_name: string;
      rgb: [number, number, number];
    }>(EVENT_MANUAL_PATCH_DISPLAYED, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      setCurrentPatch(event.payload.patch_index);
      setPatchName(event.payload.patch_name);
      setTargetRgb(event.payload.rgb);
      setMeasuredXyz(null);
      setDeltaE(null);
    });

    const unsubMeasured = listen<{
      session_id: string;
      patch_index: number;
      patch_name: string;
      target_rgb: [number, number, number];
      measured_xyz: [number, number, number];
      delta_e: number;
    }>(EVENT_MANUAL_PATCH_MEASURED, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      setMeasuredXyz(event.payload.measured_xyz);
      setDeltaE(event.payload.delta_e);
      setIsMeasuring(false);
    });

    const unsubSkipped = listen<{
      session_id: string;
      patch_index: number;
    }>(EVENT_MANUAL_PATCH_SKIPPED, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      setMeasuredXyz(null);
      setDeltaE(null);
    });

    const unsubComplete = listen<{
      session_id: string;
    }>(EVENT_MANUAL_CALIBRATION_COMPLETE, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      setIsComplete(true);
    });

    return () => {
      cancelled = true;
      unsubDisplayed.then((u) => u());
      unsubMeasured.then((u) => u());
      unsubSkipped.then((u) => u());
      unsubComplete.then((u) => u());
    };
  }, [sessionId]);

  const handleMeasure = async () => {
    if (!sessionId || isMeasuring) return;
    setIsMeasuring(true);
    try {
      await measureManualPatch(sessionId);
    } catch (e) {
      setError(String(e));
      setIsMeasuring(false);
    }
  };

  const handleNext = async () => {
    try {
      await nextManualPatch();
    } catch (e) {
      setError(String(e));
    }
  };

  const handlePrev = async () => {
    try {
      await prevManualPatch();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleSkip = async () => {
    try {
      await skipManualPatch();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleFinish = async () => {
    try {
      await finishManualCalibration(true);
      onComplete();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleAbort = async () => {
    try {
      await abortManualCalibration();
    } catch (e) {
      // ignore
    }
    onAbort();
  };

  const progress = totalPatches > 0 ? (currentPatch / totalPatches) * 100 : 0;

  return (
    <div className="space-y-6">
      {error && (
        <div className="bg-red-900/20 border border-red-800 rounded-lg p-3 text-sm text-red-400">
          {error}
        </div>
      )}

      {/* Progress */}
      <div className="flex items-center justify-between text-sm text-gray-400">
        <span>
          Patch {currentPatch + 1} of {totalPatches}
        </span>
        <span className="font-mono">{patchName}</span>
      </div>
      <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
        <div
          className="h-full bg-primary rounded-full transition-all"
          style={{ width: `${progress}%` }}
        />
      </div>

      {/* Patch display */}
      <div className="flex flex-col items-center gap-4 py-8">
        <div
          className="w-32 h-32 rounded-xl border-2 border-gray-700 shadow-lg"
          style={{
            backgroundColor: targetRgb
              ? `rgb(${Math.round(targetRgb[0] * 255)}, ${Math.round(targetRgb[1] * 255)}, ${Math.round(targetRgb[2] * 255)})`
              : "#000",
          }}
        />
        <div className="text-sm text-gray-400 font-mono">
          Target RGB: {targetRgb?.map((v) => (v * 255).toFixed(0)).join(", ") ?? "--"}
        </div>
      </div>

      {/* Measurement result */}
      {measuredXyz && (
        <div className="bg-surface-200 border border-gray-800 rounded-lg p-4 space-y-2">
          <div className="text-sm font-medium">Measurement Result</div>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <div className="text-gray-500">Measured XYZ</div>
              <div className="font-mono text-gray-300">
                {measuredXyz.map((v) => v.toFixed(2)).join(", ")}
              </div>
            </div>
            <div>
              <div className="text-gray-500">dE2000</div>
              <div
                className={`font-mono font-medium ${
                  (deltaE ?? 0) < 1
                    ? "text-green-500"
                    : (deltaE ?? 0) < 3
                    ? "text-yellow-500"
                    : "text-red-500"
                }`}
              >
                {deltaE?.toFixed(2) ?? "--"}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Patch table mini */}
      {patches.length > 0 && (
        <div className="border border-gray-800 rounded-lg overflow-hidden max-h-48 overflow-y-auto">
          <table className="w-full text-xs">
            <thead className="bg-gray-800 text-gray-400 sticky top-0">
              <tr>
                <th className="text-left px-3 py-2">#</th>
                <th className="text-left px-3 py-2">Type</th>
                <th className="text-right px-3 py-2">dE</th>
                <th className="text-center px-3 py-2">Status</th>
              </tr>
            </thead>
            <tbody>
              {patches.map((p, i) => (
                <tr
                  key={i}
                  className={`border-t border-gray-800 ${
                    i === currentPatch ? "bg-primary/10" : ""
                  }`}
                >
                  <td className="px-3 py-1.5">{i + 1}</td>
                  <td className="px-3 py-1.5">{p.patch_type}</td>
                  <td className="px-3 py-1.5 text-right font-mono">
                    {p.delta_e?.toFixed(2) ?? "--"}
                  </td>
                  <td className="px-3 py-1.5 text-center">
                    {p.skipped ? (
                      <span className="text-yellow-500">Skipped</span>
                    ) : p.measured_xyz ? (
                      <span className="text-green-500">✓</span>
                    ) : (
                      <span className="text-gray-600">—</span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Controls */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex gap-2">
          <button
            onClick={handlePrev}
            disabled={currentPatch <= 0 || isMeasuring}
            className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition disabled:opacity-50"
          >
            ← Prev
          </button>
          <button
            onClick={handleNext}
            disabled={currentPatch >= totalPatches - 1 || isMeasuring}
            className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition disabled:opacity-50"
          >
            Next →
          </button>
        </div>

        <div className="flex gap-2">
          <button
            onClick={handleSkip}
            disabled={isMeasuring}
            className="px-3 py-2 rounded-lg bg-yellow-900/20 border border-yellow-800/50 text-yellow-400 text-sm hover:bg-yellow-900/30 transition disabled:opacity-50"
          >
            Skip
          </button>
          <button
            onClick={handleMeasure}
            disabled={isMeasuring || isComplete}
            className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition disabled:opacity-50"
          >
            {isMeasuring ? "Measuring..." : "Measure"}
          </button>
        </div>

        <div className="flex gap-2">
          <button
            onClick={handleAbort}
            className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition"
          >
            Abort
          </button>
          <button
            onClick={handleFinish}
            disabled={isMeasuring}
            className="px-4 py-2 rounded-lg bg-green-600 text-white text-sm font-medium hover:bg-green-500 transition disabled:opacity-50"
          >
            Finish
          </button>
        </div>
      </div>
    </div>
  );
}
