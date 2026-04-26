import { useState } from "react";
import { ProfilingStep } from "../calibrate/ProfilingStep";

export function MeterProfiler() {
  const [meterId, setMeterId] = useState("i1-display-pro");
  const [referenceId, setReferenceId] = useState("i1-pro-2");
  const [displayId, setDisplayId] = useState("lg-oled");
  const [started, setStarted] = useState(false);

  if (!started) {
    return (
      <div className="space-y-4">
        <div className="text-sm font-medium">Profile Meter</div>
        <div className="grid grid-cols-3 gap-4">
          <select
            value={meterId}
            onChange={(e) => setMeterId(e.target.value)}
            className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm"
          >
            <option value="i1-display-pro">i1 Display Pro Rev.B</option>
          </select>
          <select
            value={referenceId}
            onChange={(e) => setReferenceId(e.target.value)}
            className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm"
          >
            <option value="i1-pro-2">i1 Pro 2</option>
          </select>
          <select
            value={displayId}
            onChange={(e) => setDisplayId(e.target.value)}
            className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm"
          >
            <option value="lg-oled">LG OLED</option>
          </select>
        </div>
        <button
          onClick={() => setStarted(true)}
          className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition"
        >
          Start Profiling
        </button>
      </div>
    );
  }

  return (
    <ProfilingStep
      meterId={meterId}
      referenceMeterId={referenceId}
      displayId={displayId}
      onComplete={(_matrix, accuracy) => {
        alert(`Profiling complete! Accuracy: ${accuracy.toFixed(2)} dE`);
        setStarted(false);
      }}
      onSkip={() => setStarted(false)}
    />
  );
}
