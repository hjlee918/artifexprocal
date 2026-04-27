import { MeterProfiler } from "../devices/MeterProfiler";

export function DevicesView() {
  return (
    <div className="space-y-6">
      <div className="text-xl font-semibold text-white">Devices</div>

      {/* Device inventory cards (placeholder for now) */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-4">
          <div className="text-sm font-medium mb-2">Connected Meters</div>
          <div className="text-gray-400 text-sm">No meters connected</div>
        </div>
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-4">
          <div className="text-sm font-medium mb-2">Connected Displays</div>
          <div className="text-gray-400 text-sm">No displays connected</div>
        </div>
      </div>

      {/* Meter Profiler */}
      <div className="bg-gray-900 border border-gray-800 rounded-xl p-4">
        <div className="text-sm font-medium mb-4">Meter Profiler</div>
        <MeterProfiler />
      </div>
    </div>
  );
}
