import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { SessionCompareView } from "../history/SessionCompareView";
import { SessionDetailDto } from "../../bindings";

function makeDetail(name: string, gamma: number, max_de: number, avg_de: number): SessionDetailDto {
  return {
    summary: {
      id: `id-${name}`,
      name,
      created_at: 1714320000000,
      ended_at: null,
      state: "finished",
      target_space: "BT.709",
      tier: "GrayscaleOnly",
      patch_count: 21,
      gamma,
      max_de,
      avg_de,
    },
    config: {
      name,
      target_space: "Bt709",
      tone_curve: "Gamma(2.2)",
      white_point: "D65",
      patch_count: 21,
      reads_per_patch: 1,
      settle_time_ms: 0,
      stability_threshold: null,
      tier: "GrayscaleOnly",
    },
    readings: [],
    results: { gamma, max_de, avg_de, white_balance: null, lut_1d_size: 256, lut_3d_size: null },
  };
}

describe("SessionCompareView", () => {
  it("renders both session names", () => {
    const a = makeDetail("Before", 2.42, 2.34, 1.12);
    const b = makeDetail("After", 2.40, 0.87, 0.45);

    render(<SessionCompareView sessionA={a} sessionB={b} onBack={vi.fn()} />);

    expect(screen.getByText("Before")).toBeInTheDocument();
    expect(screen.getByText("After")).toBeInTheDocument();
  });

  it("shows green for improved max_de", () => {
    const a = makeDetail("Before", 2.42, 2.34, 1.12);
    const b = makeDetail("After", 2.40, 0.87, 0.45);

    render(<SessionCompareView sessionA={a} sessionB={b} onBack={vi.fn()} />);

    const maxDeRow = screen.getByText("Max dE2000").closest("tr");
    expect(maxDeRow).toHaveTextContent("✓");
  });
});
