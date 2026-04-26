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
  it("renders sidebar, topbar, and footer", async () => {
    render(
      <BrowserRouter>
        <DashboardShell />
      </BrowserRouter>
    );
    expect(await screen.findByText("ArtifexProCal")).toBeInTheDocument();
    expect(await screen.findByText("Dashboard")).toBeInTheDocument();
    expect(await screen.findByText("v0.1.0-alpha")).toBeInTheDocument();
  });

  it("shows disconnected status by default", async () => {
    render(
      <BrowserRouter>
        <DashboardShell />
      </BrowserRouter>
    );
    expect(await screen.findByText("No meter")).toBeInTheDocument();
    expect(await screen.findByText("No display")).toBeInTheDocument();
  });
});
