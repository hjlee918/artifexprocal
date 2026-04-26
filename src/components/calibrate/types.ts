export type WizardStep =
  | "devices"
  | "profiling"
  | "target"
  | "measure"
  | "analyze"
  | "upload"
  | "verify";

export interface PatchReading {
  patch_index: number;
  patch_name: string;
  rgb: [number, number, number];
  yxy: [number, number, number];
  de2000: number;
}

export interface AnalysisResult {
  gamma: number;
  max_de: number;
  avg_de: number;
  white_balance_errors: [number, number, number];
}

export interface VerificationResult {
  pre_de: number[];
  post_de: number[];
}

export interface WizardState {
  step: WizardStep;
  sessionId: string | null;
  config: import("../../bindings").SessionConfigDto | null;
  readings: PatchReading[];
  analysis: AnalysisResult | null;
  verification: VerificationResult | null;
  profilingMatrix: number[][] | null;
  profilingAccuracy: number | null;
}
