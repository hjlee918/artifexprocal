import { useDashboardStore } from "../../store/useDashboardStore";
import { Plug, Monitor, Layers, Gauge, Hand } from "lucide-react";
import { useState } from "react";
import type { CalibrationMode } from "./types";

export function DeviceSelectionStep({
  onNext,
}: {
  onNext: (mode: CalibrationMode, profileFirst: boolean) => void;
}) {
  const meterStatus = useDashboardStore((s) => s.meterStatus);
  const displayStatus = useDashboardStore((s) => s.displayStatus);
  const [mode, setMode] = useState<CalibrationMode>("autocal");
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

      <div className="bg-surface-200 border border-gray-800 rounded-lg p-4 space-y-3">
        <div className="text-sm font-medium">Calibration Mode</div>
        <div className="grid grid-cols-2 gap-3">
          <ModeCard
            icon={<Gauge size={18} />}
            label="AutoCal"
            description="Automated grayscale + 3D LUT"
            selected={mode === "autocal"}
            onClick={() => setMode("autocal")}
          />
          <ModeCard
            icon={<Hand size={18} />}
            label="Manual"
            description="User-driven patch-by-patch"
            selected={mode === "manual"}
            onClick={() => setMode("manual")}
          />
        </div>
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
          onClick={() => onNext(mode, profileFirst)}
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

function ModeCard({
  icon,
  label,
  description,
  selected,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  description: string;
  selected: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex flex-col items-center gap-2 p-4 rounded-lg border transition text-left ${
        selected
          ? "bg-primary/10 border-primary text-white"
          : "bg-surface border-gray-800 text-gray-400 hover:border-gray-700"
      }`}
    >
      <div className={selected ? "text-primary" : "text-gray-500"}>{icon}</div>
      <div className="text-sm font-medium">{label}</div>
      <div className="text-xs text-gray-500">{description}</div>
    </button>
  );
}
