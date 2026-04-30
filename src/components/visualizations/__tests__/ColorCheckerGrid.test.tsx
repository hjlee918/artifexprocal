import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { ColorCheckerGrid } from "../ColorCheckerGrid";

const mockPatches = Array.from({ length: 24 }, (_, i) => ({
  measuredRgb: [0.5, 0.5, 0.5] as [number, number, number],
  de2000: i < 12 ? 0.5 : 2.5,
}));

describe("ColorCheckerGrid", () => {
  it("renders 24 patches", () => {
    const { getByTestId, container } = render(<ColorCheckerGrid patches={mockPatches} />);
    expect(getByTestId("colorchecker-grid")).toBeInTheDocument();
    const cells = container.querySelectorAll(".grid > div");
    expect(cells.length).toBe(24);
  });

  it("shows average dE", () => {
    const { container } = render(<ColorCheckerGrid patches={mockPatches} />);
    expect(container.textContent).toContain("Avg dE:");
  });
});
