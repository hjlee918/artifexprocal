import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@testing-library/react";
import { TargetConfigStep } from "../calibrate/TargetConfigStep";

describe("TargetConfigStep", () => {
  it("renders tier selector", () => {
    const { container } = render(<TargetConfigStep onStart={vi.fn()} />);
    expect(container.textContent).toContain("Calibration Tier");
  });

  it("calls onStart with selected tier", () => {
    const onStart = vi.fn();
    const { getByText, container } = render(<TargetConfigStep onStart={onStart} />);

    // Change tier to Full 3D (last select in the grid)
    const selects = container.querySelectorAll("select");
    const tierSelect = selects[selects.length - 1];
    fireEvent.change(tierSelect, { target: { value: "Full3D" } });

    fireEvent.click(getByText("Start Measurement"));
    expect(onStart).toHaveBeenCalled();
    const calledWith = onStart.mock.calls[0][0];
    expect(calledWith.tier).toBe("Full3D");
  });
});
