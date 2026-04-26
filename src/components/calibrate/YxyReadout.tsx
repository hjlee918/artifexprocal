export function YxyReadout({
  yxy,
  reads,
  stdDev,
  stable,
}: {
  yxy: [number, number, number] | null;
  reads: [number, number];
  stdDev: number;
  stable: boolean;
}) {
  return (
    <div className="grid grid-cols-2 gap-3">
      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase">Y (nits)</div>
        <div className="text-2xl font-semibold text-white">{yxy ? yxy[0].toFixed(2) : "—"}</div>
      </div>
      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase">x</div>
        <div className="text-2xl font-semibold text-white">{yxy ? yxy[1].toFixed(4) : "—"}</div>
      </div>
      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase">y</div>
        <div className="text-2xl font-semibold text-white">{yxy ? yxy[2].toFixed(4) : "—"}</div>
      </div>
      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase">Reads</div>
        <div className="text-lg font-semibold text-white">{reads[0]}/{reads[1]}</div>
        <div className={`text-xs mt-1 ${stable ? "text-green-500" : "text-yellow-500"}`}>
          {stable ? "Stable" : "Stabilizing..."}
        </div>
        <div className="text-xs text-gray-500 mt-1">SD: {stdDev.toFixed(4)}</div>
      </div>
    </div>
  );
}
