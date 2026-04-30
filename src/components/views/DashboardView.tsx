import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useDashboardStore } from "../../store/useDashboardStore";
import { Zap, Plug, Monitor, Loader2 } from "lucide-react";
import { getAppState, connectMeter, connectDisplay } from "../../bindings";

export function DashboardView() {
  const navigate = useNavigate();
  const meterStatus = useDashboardStore((s) => s.meterStatus);
  const displayStatus = useDashboardStore((s) => s.displayStatus);
  const calibrationState = useDashboardStore((s) => s.calibrationState);
  const setMeterStatus = useDashboardStore((s) => s.setMeterStatus);
  const setDisplayStatus = useDashboardStore((s) => s.setDisplayStatus);
  const [quickLoading, setQuickLoading] = useState<string | null>(null);
  const [recentError, setRecentError] = useState<string | null>(null);

  // On mount, sync device state from backend
  useEffect(() => {
    getAppState()
      .then((state) => {
        if (state.meters.length > 0) {
          const m = state.meters[0];
          setMeterStatus({ id: m.id, name: m.name, connected: m.connected, type: "meter" });
        }
        if (state.displays.length > 0) {
          const d = state.displays[0];
          setDisplayStatus({ id: d.id, name: d.name, connected: d.connected, type: "display" });
        }
      })
      .catch(() => {
        // ignore — will show disconnected
      });
  }, [setMeterStatus, setDisplayStatus]);

  const handleQuickConnectMeter = async () => {
    if (meterStatus?.connected) {
      navigate("/devices");
      return;
    }
    setQuickLoading("meter");
    setRecentError(null);
    try {
      const info = await connectMeter("i1-display-pro");
      setMeterStatus({ id: info.id, name: info.name, connected: info.connected, type: "meter" });
    } catch (e) {
      setRecentError(`Meter connect failed: ${e}`);
    } finally {
      setQuickLoading(null);
    }
  };

  const handleQuickConnectDisplay = async () => {
    if (displayStatus?.connected) {
      navigate("/devices");
      return;
    }
    setQuickLoading("display");
    setRecentError(null);
    try {
      const info = await connectDisplay("lg-oled");
      setDisplayStatus({ id: info.id, name: info.name, connected: info.connected, type: "display" });
    } catch (e) {
      setRecentError(`Display connect failed: ${e}`);
    } finally {
      setQuickLoading(null);
    }
  };

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

      {recentError && (
        <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-3 text-sm text-red-400">
          {recentError}
        </div>
      )}

      <div className="bg-surface border border-gray-800 rounded-xl p-4">
        <div className="text-sm font-medium mb-3">Quick Start</div>
        <div className="flex gap-3">
          <button
            onClick={() => navigate("/calibrate")}
            className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition flex items-center gap-2"
          >
            <Zap size={16} />
            New Calibration
          </button>
          <button
            onClick={handleQuickConnectMeter}
            disabled={quickLoading === "meter"}
            className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition flex items-center gap-2 disabled:opacity-50"
          >
            {quickLoading === "meter" ? <Loader2 size={16} className="animate-spin" /> : <Plug size={16} />}
            {meterStatus?.connected ? "Meter Connected" : "Connect Meter"}
          </button>
          <button
            onClick={handleQuickConnectDisplay}
            disabled={quickLoading === "display"}
            className="px-4 py-2 rounded-lg bg-surface-200 border border-gray-700 text-gray-300 text-sm hover:bg-surface-300 transition flex items-center gap-2 disabled:opacity-50"
          >
            {quickLoading === "display" ? <Loader2 size={16} className="animate-spin" /> : <Monitor size={16} />}
            {displayStatus?.connected ? "Display Connected" : "Connect Display"}
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
