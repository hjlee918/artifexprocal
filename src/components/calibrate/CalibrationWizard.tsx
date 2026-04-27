import type { WizardStep, WizardState } from "./types";

const steps: { key: WizardStep; label: string }[] = [
  { key: "devices", label: "Devices" },
  { key: "target", label: "Target" },
  { key: "measure", label: "Measure" },
  { key: "analyze", label: "Analyze" },
  { key: "upload", label: "Upload" },
  { key: "verify", label: "Verify" },
];

function stepIndex(step: WizardStep): number {
  if (step === "profiling") return 0;
  return steps.findIndex((s) => s.key === step);
}

export function CalibrationWizard({
  state,
  setState: _setState,
  children,
}: {
  state: WizardState;
  setState: React.Dispatch<React.SetStateAction<WizardState>>;
  children: React.ReactNode;
}) {
  const currentIdx = stepIndex(state.step);

  return (
    <div className="space-y-6">
      {/* Stepper */}
      <div className="flex items-center justify-between">
        {steps.map((s, i) => (
          <div key={s.key} className="flex items-center flex-1">
            <div className="flex flex-col items-center">
              <div
                className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
                  i <= currentIdx
                    ? "bg-primary text-white"
                    : "bg-gray-700 text-gray-500"
                }`}
              >
                {i + 1}
              </div>
              <span
                className={`text-xs mt-1 ${
                  i <= currentIdx ? "text-white" : "text-gray-500"
                }`}
              >
                {s.label}
              </span>
            </div>
            {i < steps.length - 1 && (
              <div
                className={`flex-1 h-0.5 mx-2 ${
                  i < currentIdx ? "bg-primary" : "bg-gray-800"
                }`}
              />
            )}
          </div>
        ))}
      </div>

      {/* Content */}
      <div className="bg-surface border border-gray-800 rounded-xl p-6">
        {children}
      </div>
    </div>
  );
}
