import { useEffect, useState } from "react";
import { useDashboardStore } from "../../store/useDashboardStore";
import { MeterProfiler } from "../devices/MeterProfiler";
import {
  Plug,
  Monitor,
  Link,
  Link2Off,
  Loader2,
  RefreshCw,
} from "lucide-react";
import {
  getDeviceInventory,
  getAppState,
  connectMeter,
  disconnectMeter,
  connectDisplay,
  disconnectDisplay,
} from "../../bindings";
import type { DeviceInfo, MeterInfo, DisplayInfo } from "../../bindings";

export function DevicesView() {
  const meterStatus = useDashboardStore((s) => s.meterStatus);
  const displayStatus = useDashboardStore((s) => s.displayStatus);
  const setMeterStatus = useDashboardStore((s) => s.setMeterStatus);
  const setDisplayStatus = useDashboardStore((s) => s.setDisplayStatus);

  const [inventory, setInventory] = useState<DeviceInfo[]>([]);
  const [connectedMeters, setConnectedMeters] = useState<MeterInfo[]>([]);
  const [connectedDisplays, setConnectedDisplays] = useState<DisplayInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [actionId, setActionId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setLoading(true);
    setError(null);
    try {
      const [inv, appState] = await Promise.all([
        getDeviceInventory(),
        getAppState(),
      ]);
      setInventory(inv);
      setConnectedMeters(appState.meters);
      setConnectedDisplays(appState.displays);

      // Sync store with backend state
      if (appState.meters.length > 0) {
        const m = appState.meters[0];
        setMeterStatus({ id: m.id, name: m.name, connected: m.connected, type: "meter" });
      } else {
        setMeterStatus(null);
      }
      if (appState.displays.length > 0) {
        const d = appState.displays[0];
        setDisplayStatus({ id: d.id, name: d.name, connected: d.connected, type: "display" });
      } else {
        setDisplayStatus(null);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const handleConnectMeter = async (id: string) => {
    setActionId(id);
    setError(null);
    try {
      const info = await connectMeter(id);
      setMeterStatus({ id: info.id, name: info.name, connected: info.connected, type: "meter" });
      await refresh();
    } catch (e) {
      setError(`Failed to connect meter: ${e}`);
    } finally {
      setActionId(null);
    }
  };

  const handleDisconnectMeter = async (id: string) => {
    setActionId(id);
    setError(null);
    try {
      await disconnectMeter(id);
      setMeterStatus(null);
      await refresh();
    } catch (e) {
      setError(`Failed to disconnect meter: ${e}`);
    } finally {
      setActionId(null);
    }
  };

  const handleConnectDisplay = async (id: string) => {
    setActionId(id);
    setError(null);
    try {
      const info = await connectDisplay(id);
      setDisplayStatus({ id: info.id, name: info.name, connected: info.connected, type: "display" });
      await refresh();
    } catch (e) {
      setError(`Failed to connect display: ${e}`);
    } finally {
      setActionId(null);
    }
  };

  const handleDisconnectDisplay = async (id: string) => {
    setActionId(id);
    setError(null);
    try {
      await disconnectDisplay(id);
      setDisplayStatus(null);
      await refresh();
    } catch (e) {
      setError(`Failed to disconnect display: ${e}`);
    } finally {
      setActionId(null);
    }
  };

  const availableMeters = inventory.filter((d) => d.device_type === "meter");
  const availableDisplays = inventory.filter((d) => d.device_type === "display");

  const isMeterConnected = (id: string) => connectedMeters.some((m) => m.id === id);
  const isDisplayConnected = (id: string) => connectedDisplays.some((d) => d.id === id);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="text-xl font-semibold text-white">Devices</div>
        <button
          onClick={refresh}
          disabled={loading}
          className="flex items-center gap-2 px-3 py-1.5 text-sm bg-surface-200 border border-gray-700 rounded-lg hover:bg-surface-300 transition disabled:opacity-50"
        >
          {loading ? <Loader2 size={14} className="animate-spin" /> : <RefreshCw size={14} />}
          Refresh
        </button>
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-3 text-sm text-red-400">
          {error}
        </div>
      )}

      {/* Connected Devices Summary */}
      <div className="grid grid-cols-2 gap-4">
        <DeviceStatusCard
          icon={<Plug size={18} />}
          label="Meter"
          status={meterStatus}
          onDisconnect={meterStatus ? () => handleDisconnectMeter(meterStatus.id) : undefined}
          disconnectLoading={meterStatus ? actionId === meterStatus.id : false}
        />
        <DeviceStatusCard
          icon={<Monitor size={18} />}
          label="Display"
          status={displayStatus}
          onDisconnect={displayStatus ? () => handleDisconnectDisplay(displayStatus.id) : undefined}
          disconnectLoading={displayStatus ? actionId === displayStatus.id : false}
        />
      </div>

      {/* Available Meters */}
      <div className="bg-surface border border-gray-800 rounded-xl p-4">
        <div className="text-sm font-medium mb-3 flex items-center gap-2">
          <Plug size={16} className="text-gray-500" />
          Available Meters
        </div>
        {availableMeters.length === 0 ? (
          <div className="text-sm text-gray-500">No meters found in inventory.</div>
        ) : (
          <div className="space-y-2">
            {availableMeters.map((m) => {
              const connected = isMeterConnected(m.id);
              return (
                <div
                  key={m.id}
                  className="flex items-center justify-between bg-surface-200 border border-gray-800 rounded-lg p-3"
                >
                  <div>
                    <div className="text-sm font-medium text-white">{m.name}</div>
                    <div className="text-xs text-gray-500">{m.id}</div>
                  </div>
                  {connected ? (
                    <button
                      onClick={() => handleDisconnectMeter(m.id)}
                      disabled={actionId === m.id}
                      className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-red-500/10 text-red-400 border border-red-500/20 rounded hover:bg-red-500/20 transition disabled:opacity-50"
                    >
                      {actionId === m.id ? <Loader2 size={12} className="animate-spin" /> : <Link2Off size={12} />}
                      Disconnect
                    </button>
                  ) : (
                    <button
                      onClick={() => handleConnectMeter(m.id)}
                      disabled={actionId === m.id}
                      className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-primary/10 text-primary border border-primary/20 rounded hover:bg-primary/20 transition disabled:opacity-50"
                    >
                      {actionId === m.id ? <Loader2 size={12} className="animate-spin" /> : <Link size={12} />}
                      Connect
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Available Displays */}
      <div className="bg-surface border border-gray-800 rounded-xl p-4">
        <div className="text-sm font-medium mb-3 flex items-center gap-2">
          <Monitor size={16} className="text-gray-500" />
          Available Displays
        </div>
        {availableDisplays.length === 0 ? (
          <div className="text-sm text-gray-500">No displays found in inventory.</div>
        ) : (
          <div className="space-y-2">
            {availableDisplays.map((d) => {
              const connected = isDisplayConnected(d.id);
              return (
                <div
                  key={d.id}
                  className="flex items-center justify-between bg-surface-200 border border-gray-800 rounded-lg p-3"
                >
                  <div>
                    <div className="text-sm font-medium text-white">{d.name}</div>
                    <div className="text-xs text-gray-500">{d.id}</div>
                  </div>
                  {connected ? (
                    <button
                      onClick={() => handleDisconnectDisplay(d.id)}
                      disabled={actionId === d.id}
                      className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-red-500/10 text-red-400 border border-red-500/20 rounded hover:bg-red-500/20 transition disabled:opacity-50"
                    >
                      {actionId === d.id ? <Loader2 size={12} className="animate-spin" /> : <Link2Off size={12} />}
                      Disconnect
                    </button>
                  ) : (
                    <button
                      onClick={() => handleConnectDisplay(d.id)}
                      disabled={actionId === d.id}
                      className="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-primary/10 text-primary border border-primary/20 rounded hover:bg-primary/20 transition disabled:opacity-50"
                    >
                      {actionId === d.id ? <Loader2 size={12} className="animate-spin" /> : <Link size={12} />}
                      Connect
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Meter Profiler */}
      <div className="bg-surface border border-gray-800 rounded-xl p-4">
        <div className="text-sm font-medium mb-4">Meter Profiler</div>
        <MeterProfiler />
      </div>
    </div>
  );
}

function DeviceStatusCard({
  icon,
  label,
  status,
  onDisconnect,
  disconnectLoading,
}: {
  icon: React.ReactNode;
  label: string;
  status: { id: string; name: string; connected: boolean; type: string } | null;
  onDisconnect?: () => void;
  disconnectLoading?: boolean;
}) {
  return (
    <div className="bg-surface border border-gray-800 rounded-xl p-4">
      <div className="flex items-center gap-2 text-xs text-gray-500 uppercase tracking-wider mb-2">
        {icon}
        {label}
      </div>
      {status?.connected ? (
        <div className="flex items-center justify-between">
          <div>
            <div className="font-medium text-white">{status.name}</div>
            <div className="text-xs text-green-500 mt-0.5">● Connected</div>
          </div>
          {onDisconnect && (
            <button
              onClick={onDisconnect}
              disabled={disconnectLoading}
              className="flex items-center gap-1 px-2.5 py-1.5 text-xs bg-red-500/10 text-red-400 border border-red-500/20 rounded hover:bg-red-500/20 transition disabled:opacity-50"
            >
              {disconnectLoading ? <Loader2 size={12} className="animate-spin" /> : <Link2Off size={12} />}
              Disconnect
            </button>
          )}
        </div>
      ) : (
        <div>
          <div className="text-gray-400 text-sm">Not connected</div>
          <div className="text-xs text-gray-600 mt-0.5">Select a device below to connect</div>
        </div>
      )}
    </div>
  );
}
