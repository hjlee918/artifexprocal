import { create } from "zustand";

export type Route = "/" | "/calibrate" | "/devices" | "/history" | "/settings";

interface DeviceStatus {
  id: string;
  name: string;
  connected: boolean;
  type: "meter" | "display";
}

interface DashboardState {
  sidebarExpanded: boolean;
  activeRoute: Route;
  meterStatus: DeviceStatus | null;
  displayStatus: DeviceStatus | null;
  calibrationState: string;
  lastError: string | null;
  toggleSidebar: () => void;
  setActiveRoute: (route: Route) => void;
  setMeterStatus: (status: DeviceStatus | null) => void;
  setDisplayStatus: (status: DeviceStatus | null) => void;
  setCalibrationState: (state: string) => void;
  setLastError: (error: string | null) => void;
}

export const useDashboardStore = create<DashboardState>((set) => ({
  sidebarExpanded: true,
  activeRoute: "/",
  meterStatus: null,
  displayStatus: null,
  calibrationState: "Idle",
  lastError: null,
  toggleSidebar: () => set((s) => ({ sidebarExpanded: !s.sidebarExpanded })),
  setActiveRoute: (route) => set({ activeRoute: route }),
  setMeterStatus: (status) => set({ meterStatus: status }),
  setDisplayStatus: (status) => set({ displayStatus: status }),
  setCalibrationState: (state) => set({ calibrationState: state }),
  setLastError: (error) => set({ lastError: error }),
}));
