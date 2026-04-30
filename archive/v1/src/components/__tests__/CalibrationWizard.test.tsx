import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { CalibrateView } from "../views/CalibrateView";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve("test-session-id"),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => {}),
}));

describe("CalibrateView", () => {
  it("renders device selection step by default", () => {
    render(<CalibrateView />);
    expect(screen.getByText("Meter")).toBeInTheDocument();
    expect(screen.getByText("Display")).toBeInTheDocument();
    expect(screen.getByText("Pattern Generator")).toBeInTheDocument();
  });

  it("shows pre-flight checklist", () => {
    render(<CalibrateView />);
    expect(screen.getByText("TV warmed up for 45+ minutes")).toBeInTheDocument();
  });
});
