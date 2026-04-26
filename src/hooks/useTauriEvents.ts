import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useDashboardStore } from "../store/useDashboardStore";

export function useTauriEvents() {
  const setMeterStatus = useDashboardStore((s) => s.setMeterStatus);
  const setDisplayStatus = useDashboardStore((s) => s.setDisplayStatus);
  const setCalibrationState = useDashboardStore((s) => s.setCalibrationState);
  const setLastError = useDashboardStore((s) => s.setLastError);

  useEffect(() => {
    let cancelled = false;
    const unsubs: (() => void)[] = [];

    Promise.all([
      listen("device-status-changed", (event) => {
        const payload = event.payload as {
          device_id: string;
          device_type: string;
          connected: boolean;
          info: string;
        };
        const status = {
          id: payload.device_id,
          name: payload.info,
          connected: payload.connected,
          type: payload.device_type === "display" ? "display" : "meter",
        } as const;
        if (status.type === "meter") {
          setMeterStatus(status);
        } else {
          setDisplayStatus(status);
        }
      }),
      listen("calibration-state-changed", (event) => {
        const payload = event.payload as {
          old_state: string;
          new_state: string;
          message: string;
        };
        setCalibrationState(payload.new_state);
      }),
      listen("error-occurred", (event) => {
        const payload = event.payload as {
          severity: string;
          message: string;
          source: string;
        };
        setLastError(`${payload.severity}: ${payload.message}`);
      }),
    ]).then((listeners) => {
      if (!cancelled) {
        unsubs.push(...listeners);
      } else {
        listeners.forEach((u) => u());
      }
    });

    return () => {
      cancelled = true;
      unsubs.forEach((u) => u());
    };
  }, [setMeterStatus, setDisplayStatus, setCalibrationState, setLastError]);
}
