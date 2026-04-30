import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SessionDetailView } from "../history/SessionDetailView";
import { SessionDetailDto } from "../../bindings";

const mockDetail: SessionDetailDto = {
  summary: {
    id: "test-id",
    name: "Test Session",
    created_at: 1714320000000,
    ended_at: null,
    state: "finished",
    target_space: "BT.709",
    tier: "GrayscaleOnly",
    patch_count: 2,
    gamma: 2.4,
    max_de: 1.23,
    avg_de: 0.45,
  },
  config: {
    name: "Test Session",
    target_space: "Bt709",
    tone_curve: "Gamma(2.2)",
    white_point: "D65",
    patch_count: 2,
    reads_per_patch: 1,
    settle_time_ms: 0,
    stability_threshold: null,
    tier: "GrayscaleOnly",
  },
  readings: [
    {
      patch_index: 0,
      target_rgb: [0, 0, 0],
      measured_xyz: [0.5, 0.5, 0.5],
      reading_index: 0,
      measurement_type: "cal",
    },
  ],
  results: {
    gamma: 2.4,
    max_de: 1.23,
    avg_de: 0.45,
    white_balance: null,
    lut_1d_size: 256,
    lut_3d_size: null,
  },
};

describe("SessionDetailView", () => {
  it("renders summary cards", () => {
    render(
      <SessionDetailView
        detail={mockDetail}
        onBack={vi.fn()}
        onExport={vi.fn()}
        onCompare={vi.fn()}
      />
    );

    expect(screen.getByText("Test Session")).toBeInTheDocument();
    expect(screen.getByText("2.40")).toBeInTheDocument();
  });

  it("switches to readings tab", () => {
    render(
      <SessionDetailView
        detail={mockDetail}
        onBack={vi.fn()}
        onExport={vi.fn()}
        onCompare={vi.fn()}
      />
    );

    fireEvent.click(screen.getByText("readings"));
    expect(screen.getByText("Patch")).toBeInTheDocument();
  });

  it("calls onBack when back clicked", () => {
    const onBack = vi.fn();
    render(
      <SessionDetailView
        detail={mockDetail}
        onBack={onBack}
        onExport={vi.fn()}
        onCompare={vi.fn()}
      />
    );

    fireEvent.click(screen.getByText("← Back to History"));
    expect(onBack).toHaveBeenCalled();
  });
});
