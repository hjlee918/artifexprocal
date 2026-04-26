import { Bell, Octagon } from "lucide-react";
import { useDashboardStore } from "../store/useDashboardStore";

function StatusDot({ connected }: { connected: boolean }) {
  return (
    <span
      className={`inline-block w-2 h-2 rounded-full ${
        connected ? "bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.4)]" : "bg-gray-600"
      }`}
    />
  );
}

export function TopBar() {
  const { meterStatus, displayStatus, calibrationState, activeRoute } =
    useDashboardStore();

  const routeTitles: Record<string, string> = {
    "/": "Dashboard",
    "/calibrate": "Calibrate",
    "/devices": "Devices",
    "/history": "History",
    "/settings": "Settings",
  };

  return (
    <header className="h-12 bg-surface border-b border-gray-800 flex items-center justify-between px-4 shrink-0">
      <div className="text-sm font-medium text-white">
        {routeTitles[activeRoute] ?? "ArtifexProCal"}
      </div>

      <div className="flex items-center gap-4 text-xs text-gray-400">
        <div className="flex items-center gap-1.5">
          <StatusDot connected={meterStatus?.connected ?? false} />
          <span>{meterStatus?.name ?? "No meter"}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <StatusDot connected={displayStatus?.connected ?? false} />
          <span>{displayStatus?.name ?? "No display"}</span>
        </div>
        <div className="px-2 py-0.5 rounded bg-surface-200 text-gray-300 border border-gray-700">
          {calibrationState}
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button className="p-1.5 rounded hover:bg-surface-200 text-gray-400 hover:text-white transition">
          <Bell size={18} />
        </button>
        <button className="px-2 py-1 rounded bg-red-500/10 text-red-500 text-xs font-medium hover:bg-red-500/20 transition">
          <Octagon size={14} className="inline mr-1" />
          STOP
        </button>
      </div>
    </header>
  );
}
