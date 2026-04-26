import { useCallback, useEffect } from "react";
import {
  getAppState,
  type AppState,
  type MeterInfo,
  type DisplayInfo,
} from "../bindings";
import { useDashboardStore } from "../store/useDashboardStore";

export function useAppState() {
  const setMeterStatus = useDashboardStore((s) => s.setMeterStatus);
  const setDisplayStatus = useDashboardStore((s) => s.setDisplayStatus);
  const setCalibrationState = useDashboardStore((s) => s.setCalibrationState);
  const setLastError = useDashboardStore((s) => s.setLastError);

  const refresh = useCallback(async () => {
    try {
      const state = await getAppState();
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
