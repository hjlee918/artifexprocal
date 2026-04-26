import { useDashboardStore } from "../../store/useDashboardStore";
import { Zap, Plug, Monitor } from "lucide-react";

export function DashboardView() {
  const meterStatus = useDashboardStore((s) => s.meterStatus);
  const displayStatus = useDashboardStore((s) => s.displayStatus);
  const calibrationState = useDashboardStore((s) => s.calibrationState);

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-surface border border-gray-800 rounded-xl p-4">
          <div className="text-xs text-gray-500 uppercase tracking-wider mb-1">Meter Status</div>
          <div className="text-2xl font-semibold">
            {meterStatus?.connected ? meterStatus.name : "Disconnected"}
          </div>
          <div className="text-xs text-gray-500 mt-1">
            {meterStatus?.connected ? "Ready for measurement" : "Connect a meter to begin"}
          </div>
        </div>
        <div className="bg-surface border border-gray-800 rounded-xl p-4">
          <div className="text-xs text-gray-500 uppercase tracking-wider mb-1">Display Status</div>
          <div className="text-2xl font-semibold">
            {displayStatus?.connected ? displayStatus.name : "Disconnected"}
          </div>
          <div className="text-xs text-gray-500 mt-1">
            {displayStatus?.connected ? "Ready for calibration" : "Connect a display"}
          </div>
        </div>
        <div className="bg-surface border border-gray-800 rounded-xl p-4">
          <div className="text-xs text-gray-500 uppercase tracking-wider mb-1">Calibration State</div>
          <div className="text-2xl font-semibold text-primary">{calibrationState}</div>
          <div className="text-xs text-gray-500 mt-1">Current session state</div>
        </div>
      </div>

      <div className="bg-surface border border-gray-800 rounded-xl p-4">
        <div className="text-sm font-medium mb-3">Quick Start</div>
        <div className="flex gap-3">
          <button className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition flex items-center gap-2">
            <Zap size={16} />
            New Calibration
          </button>
          <button className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition flex items-center gap-2">
            <Plug size={16} />
            Connect Meter
          </button>
          <button className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition flex items-center gap-2">
            <Monitor size={16} />
            Connect Display
          </button>
        </div>
      </div>

      <div className="bg-surface border border-gray-800 rounded-xl overflow-hidden">
        <div className="px-4 py-3 border-b border-gray-800 text-sm font-medium">Recent Sessions</div>
        <div className="px-4 py-8 text-center text-gray-500 text-sm">
          No calibration sessions yet. Start your first calibration above.
        </div>
      </div>
    </div>
  );
}
