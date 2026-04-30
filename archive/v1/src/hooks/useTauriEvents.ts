import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  EVENT_DEVICE_STATUS_CHANGED,
  EVENT_CALIBRATION_STATE_CHANGED,
  EVENT_ERROR_OCCURRED,
} from "../bindings";

interface DeviceStatusEvent {
  device_id: string;
  device_type: string;
  connected: boolean;
  info: string;
}

interface CalibrationStateEvent {
  old_state: string;
  new_state: string;
  message: string;
}

interface ErrorEvent {
  severity: string;
  message: string;
  source: string;
}
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
      listen<DeviceStatusEvent>(EVENT_DEVICE_STATUS_CHANGED, (event) => {
        const payload = event.payload;
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
      listen<CalibrationStateEvent>(EVENT_CALIBRATION_STATE_CHANGED, (event) => {
        setCalibrationState(event.payload.new_state);
      }),
      listen<ErrorEvent>(EVENT_ERROR_OCCURRED, (event) => {
        const p = event.payload;
        setLastError(`${p.severity}: ${p.message}`);
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
