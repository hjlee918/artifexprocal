import { useDashboardStore } from "../../store/useDashboardStore";
import { Plug, Monitor, Layers } from "lucide-react";
import { useState } from "react";

export function DeviceSelectionStep({
  onNext,
}: {
  onNext: (profileFirst: boolean) => void;
}) {
  const meterStatus = useDashboardStore((s) => s.meterStatus);
  const displayStatus = useDashboardStore((s) => s.displayStatus);
  const [profileFirst, setProfileFirst] = useState(false);

  const allConnected = meterStatus?.connected && displayStatus?.connected;

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-3 gap-4">
        <DeviceCard
          icon={<Plug size={20} />}
          label="Meter"
          name={meterStatus?.name ?? "Not connected"}
          connected={meterStatus?.connected ?? false}
        />
        <DeviceCard
          icon={<Monitor size={20} />}
          label="Display"
          name={displayStatus?.name ?? "Not connected"}
          connected={displayStatus?.connected ?? false}
        />
        <DeviceCard
          icon={<Layers size={20} />}
          label="Pattern Generator"
          name="iTPG (Internal)"
          connected={true}
        />
      </div>

      <div className="bg-surface-200 border border-gray-800 rounded-lg p-4 space-y-3">
        <div className="text-sm font-medium">Pre-flight Checklist</div>
        <ChecklistItem label="TV warmed up for 45+ minutes" checked={true} />
        <ChecklistItem label="Meter initialized (20+ min USB warm-up)" checked={meterStatus?.connected ?? false} />
        <ChecklistItem label="HDR blank video playing (for HDR mode)" checked={false} optional />
      </div>

      <label className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
        <input
          type="checkbox"
          checked={profileFirst}
          onChange={(e) => setProfileFirst(e.target.checked)}
          className="rounded border-gray-700 bg-surface-200"
        />
        Profile connected colorimeter first (requires spectrophotometer reference)
      </label>

      <div className="flex justify-end">
        <button
          onClick={() => onNext(profileFirst)}
          disabled={!allConnected}
          className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Next: Target Config
        </button>
      </div>
    </div>
  );
}

function DeviceCard({
  icon,
  label,
  name,
  connected,
}: {
  icon: React.ReactNode;
  label: string;
  name: string;
  connected: boolean;
}) {
  return (
    <div className="bg-surface-200 border border-gray-800 rounded-lg p-4">
      <div className="flex items-center gap-2 text-xs text-gray-500 uppercase tracking-wider mb-2">
        {icon}
        {label}
      </div>
      <div className="font-medium text-white">{name}</div>
      <div className={`text-xs mt-1 ${connected ? "text-green-500" : "text-red-500"}`}>
        {connected ? "● Connected" : "● Not connected"}
      </div>
    </div>
  );
}

function ChecklistItem({
  label,
  checked,
  optional,
}: {
  label: string;
  checked: boolean;
  optional?: boolean;
}) {
  return (
    <div className="flex items-center gap-2 text-sm">
      <span className={checked ? "text-green-500" : "text-gray-600"}>
        {checked ? "✓" : optional ? "○" : "✗"}
      </span>
      <span className={checked ? "text-gray-300" : "text-gray-500"}>{label}</span>
      {optional && <span className="text-xs text-gray-600">(optional)</span>}
    </div>
  );
}
