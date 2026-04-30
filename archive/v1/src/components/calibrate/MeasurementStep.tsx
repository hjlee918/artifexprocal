import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { LiveGammaChart } from "./LiveGammaChart";
import { PatchDataTable } from "./PatchDataTable";
import { YxyReadout } from "./YxyReadout";
import type { PatchReading } from "./types";
import { EVENT_CALIBRATION_PROGRESS } from "../../bindings";

interface CalibrationProgress {
  session_id: string;
  current_patch: number;
  total_patches: number;
  patch_name: string;
  yxy: [number, number, number] | null;
  stable: boolean;
}

export function MeasurementStep({
  sessionId,
  totalPatches,
  onComplete: _onComplete,
}: {
  sessionId: string;
  totalPatches: number;
  onComplete: (readings: PatchReading[]) => void;
}) {
  const [currentPatch, setCurrentPatch] = useState(0);
  const [patchName, setPatchName] = useState("Starting...");
  const [yxy, setYxy] = useState<[number, number, number] | null>(null);
  const [stable, setStable] = useState(false);
  const [readings, setReadings] = useState<PatchReading[]>([]);

  useEffect(() => {
    let cancelled = false;
    const unsubPromise = listen<CalibrationProgress>(EVENT_CALIBRATION_PROGRESS, (event) => {
      if (event.payload.session_id !== sessionId || cancelled) return;
      const p = event.payload;
      setCurrentPatch(p.current_patch);
      setPatchName(p.patch_name);
      if (p.yxy) setYxy(p.yxy);
      setStable(p.stable);

      if (p.current_patch > 0 && p.stable) {
        const level = p.current_patch / p.total_patches;
        const newReading: PatchReading = {
          patch_index: p.current_patch,
          patch_name: p.patch_name,
          rgb: [level, level, level],
          yxy: p.yxy ?? [0, 0, 0],
          de2000: 0, // Computed by backend in analysis step
        };
        setReadings((prev) => {
          const filtered = prev.filter((r) => r.patch_index !== p.current_patch);
          return [...filtered, newReading];
        });
      }
    });

    return () => {
      cancelled = true;
      unsubPromise.then((u) => u());
    };
  }, [sessionId]);

  const progress = totalPatches > 0 ? (currentPatch / totalPatches) * 100 : 0;
  const gammaPoints = readings.map((r) => ({ level: (r.patch_index / totalPatches) * 100, y: r.yxy[0] }));

  return (
    <div className="space-y-4">
      {/* Progress */}
      <div>
        <div className="flex justify-between text-xs text-gray-400 mb-1">
          <span>0%</span>
          <span>
            Patch {currentPatch} of {totalPatches} — {patchName}
          </span>
          <span>100%</span>
        </div>
        <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
          <div
            className="h-full bg-primary rounded-full transition-all duration-300"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>

      {/* Chart + Readout */}
      <div className="flex gap-4">
        <div className="flex-[2] bg-gray-800 border border-gray-800 rounded-lg p-3">
          <div className="text-xs text-gray-500 uppercase mb-2">Gamma Curve</div>
          <LiveGammaChart targetGamma={2.4} measuredPoints={gammaPoints} />
        </div>
        <div className="flex-1">
          <YxyReadout yxy={yxy} reads={[3, 5]} stdDev={0.02} stable={stable} />
        </div>
      </div>

      {/* Table */}
      <PatchDataTable readings={readings} />

      {/* Controls */}
      <div className="flex justify-center gap-3">
        <button className="px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition">
          Pause
        </button>
        <button className="px-4 py-2 rounded-lg bg-red-500/10 text-red-500 text-sm hover:bg-red-500/20 transition">
          Stop
        </button>
      </div>
    </div>
  );
}
