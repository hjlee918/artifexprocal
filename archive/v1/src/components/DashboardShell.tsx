import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";
import { StatusFooter } from "./StatusFooter";
import { useTauriEvents } from "../hooks/useTauriEvents";
import { useAppState } from "../hooks/useAppState";

export function DashboardShell() {
  useTauriEvents();
  useAppState();

  return (
    <div className="h-screen flex overflow-hidden bg-background">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <TopBar />
        <main className="flex-1 overflow-auto p-6">
          <Outlet />
        </main>
        <StatusFooter />
      </div>
    </div>
  );
}
