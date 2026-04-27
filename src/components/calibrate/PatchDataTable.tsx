import type { PatchReading } from "./types";

export function PatchDataTable({ readings }: { readings: PatchReading[] }) {
  return (
    <div className="border border-gray-800 rounded-lg overflow-hidden max-h-48 overflow-y-auto">
      <table className="w-full text-xs">
        <thead className="bg-gray-800 text-gray-400 sticky top-0">
          <tr>
            <th className="text-left px-3 py-2">Patch</th>
            <th className="text-right px-3 py-2">Y (nits)</th>
            <th className="text-right px-3 py-2">x</th>
            <th className="text-right px-3 py-2">y</th>
            <th className="text-right px-3 py-2">dE2000</th>
          </tr>
        </thead>
        <tbody>
          {readings.map((r) => {
            const deColor = r.de2000 < 1 ? "text-green-500" : r.de2000 < 3 ? "text-yellow-500" : "text-red-500";
            return (
              <tr key={r.patch_index} className="border-t border-gray-800 hover:bg-gray-800/50">
                <td className="px-3 py-1.5">{r.patch_name}</td>
                <td className="px-3 py-1.5 text-right">{r.yxy[0].toFixed(2)}</td>
                <td className="px-3 py-1.5 text-right">{r.yxy[1].toFixed(4)}</td>
                <td className="px-3 py-1.5 text-right">{r.yxy[2].toFixed(4)}</td>
                <td className={`px-3 py-1.5 text-right font-medium ${deColor}`}>{r.de2000.toFixed(2)}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
