import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { CalibrationWizard } from "../calibrate/CalibrationWizard";
import { DeviceSelectionStep } from "../calibrate/DeviceSelectionStep";
import { TargetConfigStep } from "../calibrate/TargetConfigStep";
import { MeasurementStep } from "../calibrate/MeasurementStep";
import { ManualCalibrationStep } from "../calibrate/ManualCalibrationStep";
import { AnalysisStep } from "../calibrate/AnalysisStep";
import { UploadStep } from "../calibrate/UploadStep";
import { VerifyStep } from "../calibrate/VerifyStep";
import type { WizardState, PatchReading, VerificationResult } from "../calibrate/types";
import {
  startCalibration,
  EVENT_ANALYSIS_COMPLETE,
  EVENT_LUT3D_DATA,
} from "../../bindings";

export function CalibrateView() {
  const [state, setState] = useState<WizardState>({
    step: "devices",
    mode: "autocal",
    sessionId: null,
    config: null,
    manualConfig: null,
    readings: [],
    analysis: null,
    verification: null,
    lut3d: null,
    profilingMatrix: null,
    profilingAccuracy: null,
  });

  const handleDeviceNext = async (mode: import("../calibrate/types").CalibrationMode, profileFirst: boolean) => {
    if (profileFirst) {
      setState((s) => ({ ...s, mode, step: "profiling" }));
      return;
    }
    setState((s) => ({ ...s, mode, step: "target" }));
  };

  const handleStartMeasurement = async (config: import("../../bindings").SessionConfigDto) => {
    try {
      if (state.mode === "manual") {
        const manualConfig: import("../../bindings").ManualConfigDto = {
          name: config.name,
          target_space: config.target_space,
          tone_curve: config.tone_curve,
          white_point: config.white_point,
          patch_set: "grayscale",
          custom_patches: null,
          reads_per_patch: config.reads_per_patch,
          settle_time_ms: config.settle_time_ms,
          stability_threshold: config.stability_threshold,
        };
        setState((s) => ({ ...s, step: "measure", sessionId: null, config, manualConfig }));
      } else {
        const sessionId = await startCalibration(config);
        setState((s) => ({ ...s, step: "measure", sessionId, config }));
      }
    } catch (e) {
      console.error("Failed to start calibration:", e);
    }
  };

  useEffect(() => {
    let cancelled = false;
    const unsubAnalysis = listen<{
      session_id: string;
      gamma: number;
      max_de: number;
      avg_de: number;
      white_balance_errors: number[];
    }>(EVENT_ANALYSIS_COMPLETE, (event) => {
      if (cancelled) return;
      setState((s) => ({
        ...s,
        step: "analyze",
        analysis: {
          gamma: event.payload.gamma,
          max_de: event.payload.max_de,
          avg_de: event.payload.avg_de,
          white_balance_errors: [
            event.payload.white_balance_errors[0] ?? 0,
            event.payload.white_balance_errors[1] ?? 0,
            event.payload.white_balance_errors[2] ?? 0,
          ] as [number, number, number],
        },
      }));
    });

    const unsubLut3d = listen<{
      session_id: string;
      size: number;
      data: number[];
    }>(EVENT_LUT3D_DATA, (event) => {
      if (cancelled) return;
      setState((s) => ({
        ...s,
        lut3d: {
          size: event.payload.size,
          data: event.payload.data,
        },
      }));
    });

    return () => {
      cancelled = true;
      unsubAnalysis.then((u) => u());
      unsubLut3d.then((u) => u());
    };
  }, []);

  const handleMeasurementComplete = (_readings: PatchReading[]) => {
    // Analysis is now driven by backend analysis-complete event
  };

  const handleApplyCorrections = () => {
    setState((s) => ({ ...s, step: "upload" }));
  };

  const handleUploadComplete = () => {
    const verification: VerificationResult = {
      pre_de: state.readings.map((r) => r.de2000),
      post_de: state.readings.map((r) => Math.max(0, r.de2000 * 0.3)),
    };
    setState((s) => ({ ...s, step: "verify", verification }));
  };

  const handleSaveSession = () => {
    // TODO: Save to SQLite via calibration-storage
    alert("Session saved!");
  };

  return (
    <div className="p-6">
      <CalibrationWizard state={state} setState={setState}>
        {state.step === "devices" && (
          <DeviceSelectionStep onNext={handleDeviceNext} />
        )}
        {state.step === "profiling" && (
          <div className="text-center py-12 text-gray-400">
            Profiling step placeholder — implement in Task 16
          </div>
        )}
        {state.step === "target" && (
          <TargetConfigStep onStart={handleStartMeasurement} />
        )}
        {state.step === "measure" && state.mode === "autocal" && state.sessionId && (
          <MeasurementStep
            sessionId={state.sessionId}
            totalPatches={state.config?.patch_count ?? 21}
            onComplete={handleMeasurementComplete}
          />
        )}
        {state.step === "measure" && state.mode === "manual" && state.manualConfig && (
          <ManualCalibrationStep
            config={state.manualConfig}
            onComplete={() => setState((s) => ({ ...s, step: "analyze" }))}
            onAbort={() => setState((s) => ({ ...s, step: "devices" }))}
          />
        )}
        {state.step === "analyze" && state.analysis && (
          <AnalysisStep
            readings={state.readings}
            analysis={state.analysis}
            targetSpace={state.config?.target_space}
            onApply={handleApplyCorrections}
            onRemeasure={() => setState((s) => ({ ...s, step: "target" }))}
            tier={state.config?.tier}
            lut3d={state.lut3d}
          />
        )}
        {state.step === "upload" && (
          <UploadStep onComplete={handleUploadComplete} />
        )}
        {state.step === "verify" && state.verification && (
          <VerifyStep result={state.verification} onSave={handleSaveSession} />
        )}
      </CalibrationWizard>
    </div>
  );
}
