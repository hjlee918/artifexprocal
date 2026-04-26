import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useDashboardStore } from "../store/useDashboardStore";

export function useTauriEvents() {
  const {
    setMeterStatus,
    setDisplayStatus,
    setCalibrationState,
    setLastError,
  } = useDashboardStore();

  useEffect(() => {
    const unsubs: (() => void)[] = [];

    listen("device-status-changed", (event) => {
      const payload = event.payload as {
        device_id: string;
        connected: boolean;
        info: string;
      };
      const status = {
        id: payload.device_id,
        name: payload.info,
        connected: payload.connected,
        type: payload.device_id.includes("display") ? "display" : "meter",
      } as const;
      if (status.type === "meter") {
        setMeterStatus(status);
      } else {
        setDisplayStatus(status);
      }
    }).then((unsub) => unsubs.push(unsub));

    listen("calibration-state-changed", (event) => {
      const payload = event.payload as {
        old_state: string;
        new_state: string;
        message: string;
      };
      setCalibrationState(payload.new_state);
    }).then((unsub) => unsubs.push(unsub));

    listen("error-occurred", (event) => {
      const payload = event.payload as {
        severity: string;
        message: string;
        source: string;
      };
      setLastError(`${payload.severity}: ${payload.message}`);
    }).then((unsub) => unsubs.push(unsub));

    return () => {
      unsubs.forEach((u) => u());
    };
  }, [setMeterStatus, setDisplayStatus, setCalibrationState, setLastError]);
}
