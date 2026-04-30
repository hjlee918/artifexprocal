import { useState } from "react";
import type { SessionConfigDto } from "../../bindings";

const TARGET_SPACES = ["Rec.709", "Rec.2020", "DCI-P3"];
const TONE_CURVES = ["Gamma 2.2", "Gamma 2.4", "BT.1886", "PQ", "HLG"];
const WHITE_POINTS = ["D65", "D50", "DCI"];
const PATCH_COUNTS = [21, 33, 51];
const READS_PER_PATCH = [3, 5, 10];
const SETTLE_TIMES = [
  { label: "0.5s", value: 500 },
  { label: "1s", value: 1000 },
  { label: "2s", value: 2000 },
  { label: "5s", value: 5000 },
];
const TIERS = [
  { label: "Grayscale Only", value: "GrayscaleOnly" },
  { label: "Grayscale + 3D LUT", value: "GrayscalePlus3D" },
  { label: "Full 3D LUT", value: "Full3D" },
];

export function TargetConfigStep({
  onStart,
}: {
  onStart: (config: SessionConfigDto) => void;
}) {
  const [config, setConfig] = useState<SessionConfigDto>({
    name: "Greyscale AutoCal",
    target_space: "Rec.709",
    tone_curve: "Gamma 2.4",
    white_point: "D65",
    patch_count: 21,
    reads_per_patch: 5,
    settle_time_ms: 1000,
    stability_threshold: null,
    tier: "GrayscaleOnly",
  });

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-2 gap-4">
        <SelectField
          label="Color Space"
          value={config.target_space}
          options={TARGET_SPACES}
          onChange={(v) => setConfig((c) => ({ ...c, target_space: v }))}
        />
        <SelectField
          label="Tone Curve"
          value={config.tone_curve}
          options={TONE_CURVES}
          onChange={(v) => setConfig((c) => ({ ...c, tone_curve: v }))}
        />
        <SelectField
          label="White Point"
          value={config.white_point}
          options={WHITE_POINTS}
          onChange={(v) => setConfig((c) => ({ ...c, white_point: v }))}
        />
        <SelectField
          label="Patch Count"
          value={String(config.patch_count)}
          options={PATCH_COUNTS.map(String)}
          onChange={(v) => setConfig((c) => ({ ...c, patch_count: Number(v) }))}
        />
        <SelectField
          label="Reads Per Patch"
          value={String(config.reads_per_patch)}
          options={READS_PER_PATCH.map(String)}
          onChange={(v) => setConfig((c) => ({ ...c, reads_per_patch: Number(v) }))}
        />
        <SelectField
          label="Settle Time"
          value={String(config.settle_time_ms)}
          options={SETTLE_TIMES.map((s) => String(s.value))}
          optionLabels={SETTLE_TIMES.map((s) => s.label)}
          onChange={(v) => setConfig((c) => ({ ...c, settle_time_ms: Number(v) }))}
        />
        <SelectField
          label="Calibration Tier"
          value={config.tier}
          options={TIERS.map((t) => t.value)}
          optionLabels={TIERS.map((t) => t.label)}
          onChange={(v) => setConfig((c) => ({ ...c, tier: v }))}
        />
      </div>

      <div className="flex justify-between">
        <button className="px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-gray-300 text-sm hover:bg-gray-700 transition">
          Back
        </button>
        <button
          onClick={() => onStart(config)}
          className="px-4 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-sky-400 transition"
        >
          Start Measurement
        </button>
      </div>
    </div>
  );
}

function SelectField({
  label,
  value,
  options,
  optionLabels,
  onChange,
}: {
  label: string;
  value: string;
  options: string[];
  optionLabels?: string[];
  onChange: (value: string) => void;
}) {
  return (
    <div>
      <label className="block text-xs text-gray-500 uppercase tracking-wider mb-1">{label}</label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white focus:outline-none focus:border-primary"
      >
        {options.map((opt, i) => (
          <option key={opt} value={opt}>
            {optionLabels?.[i] ?? opt}
          </option>
        ))}
      </select>
    </div>
  );
}
