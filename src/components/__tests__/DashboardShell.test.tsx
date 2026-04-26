import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { DashboardShell } from "../DashboardShell";

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () =>
    Promise.resolve({
      meters: [],
      displays: [],
      calibration_state: "Idle",
      last_error: null,
    }),
}));

describe("DashboardShell", () => {
  it("renders sidebar, topbar, and footer", () => {
    render(
      <BrowserRouter>
        <DashboardShell />
      </BrowserRouter>
    );
    expect(screen.getByText("ArtifexProCal")).toBeInTheDocument();
    expect(screen.getByText("Dashboard")).toBeInTheDocument();
    expect(screen.getByText("v0.1.0-alpha")).toBeInTheDocument();
  });

  it("shows disconnected status by default", () => {
    render(
      <BrowserRouter>
        <DashboardShell />
      </BrowserRouter>
    );
    expect(screen.getByText("No meter")).toBeInTheDocument();
    expect(screen.getByText("No display")).toBeInTheDocument();
  });
});
