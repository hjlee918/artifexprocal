import { Routes, Route, useLocation } from "react-router-dom";
import { DashboardShell } from "./components/DashboardShell";
import { DashboardView } from "./components/views/DashboardView";
import { CalibrateView } from "./components/views/CalibrateView";
import { DevicesView } from "./components/views/DevicesView";
import { HistoryView } from "./components/views/HistoryView";
import { SettingsView } from "./components/views/SettingsView";
import { useDashboardStore } from "./store/useDashboardStore";
import { useEffect } from "react";

function RouteTracker() {
  const location = useLocation();
  const setActiveRoute = useDashboardStore((s) => s.setActiveRoute);

  useEffect(() => {
    setActiveRoute(location.pathname as "/" | "/calibrate" | "/devices" | "/history" | "/settings");
  }, [location, setActiveRoute]);

  return null;
}

function App() {
  return (
    <>
      <RouteTracker />
      <Routes>
        <Route element={<DashboardShell />}>
          <Route path="/" element={<DashboardView />} />
          <Route path="/calibrate" element={<CalibrateView />} />
          <Route path="/devices" element={<DevicesView />} />
          <Route path="/history" element={<HistoryView />} />
          <Route path="/settings" element={<SettingsView />} />
        </Route>
      </Routes>
    </>
  );
}

export default App;
