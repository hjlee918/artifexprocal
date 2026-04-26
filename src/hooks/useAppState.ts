import { useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useDashboardStore } from "../store/useDashboardStore";

interface MeterInfo {
  id: string;
  name: string;
  serial: string | null;
  connected: boolean;
  capabilities: string[];
}

interface DisplayInfo {
  id: string;
  name: string;
  model: string;
  connected: boolean;
  picture_mode: string | null;
}

interface AppState {
  meters: MeterInfo[];
  displays: DisplayInfo[];
  calibration_state: string;
  last_error: string | null;
}

export function useAppState() {
  const setMeterStatus = useDashboardStore((s) => s.setMeterStatus);
  const setDisplayStatus = useDashboardStore((s) => s.setDisplayStatus);
  const setCalibrationState = useDashboardStore((s) => s.setCalibrationState);
  const setLastError = useDashboardStore((s) => s.setLastError);

  const refresh = useCallback(async () => {
    try {
      const state = await invoke<AppState>("get_app_state");
      if (state.meters.length > 0) {
        const m = state.meters[0];
        setMeterStatus({
          id: m.id,
          name: m.name,
          connected: m.connected,
          type: "meter",
        });
      } else {
        setMeterStatus(null);
      }
      if (state.displays.length > 0) {
        const d = state.displays[0];
        setDisplayStatus({
          id: d.id,
          name: d.name,
          connected: d.connected,
          type: "display",
        });
      } else {
        setDisplayStatus(null);
      }
      setCalibrationState(state.calibration_state);
      setLastError(state.last_error);
    } catch (e) {
      setLastError(String(e));
    }
  }, [setMeterStatus, setDisplayStatus, setCalibrationState, setLastError]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { refresh };
}
