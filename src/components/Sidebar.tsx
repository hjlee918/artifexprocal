import {
  LayoutDashboard,
  Sliders,
  Monitor,
  History,
  Settings,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import { useDashboardStore } from "../store/useDashboardStore";
import type { Route } from "../store/useDashboardStore";
import { Link } from "react-router-dom";

const navItems: { route: Route; label: string; icon: React.ReactNode }[] = [
  { route: "/", label: "Dashboard", icon: <LayoutDashboard size={20} /> },
  { route: "/calibrate", label: "Calibrate", icon: <Sliders size={20} /> },
  { route: "/devices", label: "Devices", icon: <Monitor size={20} /> },
  { route: "/history", label: "History", icon: <History size={20} /> },
  { route: "/settings", label: "Settings", icon: <Settings size={20} /> },
];

export function Sidebar() {
  const sidebarExpanded = useDashboardStore((s) => s.sidebarExpanded);
  const toggleSidebar = useDashboardStore((s) => s.toggleSidebar);
  const activeRoute = useDashboardStore((s) => s.activeRoute);

  return (
    <aside
      className="flex flex-col bg-surface border-r border-gray-800 transition-all duration-200"
      style={{ width: sidebarExpanded ? 240 : 64 }}
    >
      <div className="h-12 flex items-center px-4 border-b border-gray-800">
        <div className="w-7 h-7 rounded bg-primary flex items-center justify-center text-white font-bold text-xs">
          A
        </div>
        {sidebarExpanded && (
          <span className="ml-3 font-semibold text-white text-sm tracking-tight truncate">
            ArtifexProCal
          </span>
        )}
      </div>

      <nav className="flex-1 py-3 px-2 space-y-1">
        {navItems.map((item) => (
          <Link
            key={item.route}
            to={item.route}
            className={`
              flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition
              ${activeRoute === item.route ? "text-white bg-surface-200 border-l-2 border-primary" : "text-gray-400 hover:text-white hover:bg-surface-200"}
            `}
          >
            {item.icon}
            {sidebarExpanded && <span>{item.label}</span>}
          </Link>
        ))}
      </nav>

      <div className="p-2 border-t border-gray-800">
        <button
          onClick={toggleSidebar}
          className="w-full flex items-center justify-center py-2 rounded-lg hover:bg-surface-200 text-gray-400 hover:text-white transition"
        >
          {sidebarExpanded ? <ChevronLeft size={18} /> : <ChevronRight size={18} />}
        </button>
      </div>
    </aside>
  );
}
