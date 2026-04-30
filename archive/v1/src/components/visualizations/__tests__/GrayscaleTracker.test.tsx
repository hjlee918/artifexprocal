import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { GrayscaleTracker } from "../GrayscaleTracker";

const mockPoints = [
  { level: 0, r: 0, g: 0, b: 0, y: 0.1, de: 0.2, x: 0.31, y_chromaticity: 0.33 },
  { level: 50, r: 0.5, g: 0.5, b: 0.5, y: 18, de: 1.1, x: 0.32, y_chromaticity: 0.34 },
  { level: 100, r: 1, g: 1, b: 1, y: 100, de: 0.5, x: 0.31, y_chromaticity: 0.33 },
];

describe("GrayscaleTracker", () => {
  it("renders svg", () => {
    const { getByTestId } = render(
      <GrayscaleTracker points={mockPoints} targetGamma={2.4} />
    );
    expect(getByTestId("grayscale-tracker")).toBeInTheDocument();
    expect(getByTestId("grayscale-tracker").tagName).toBe("svg");
  });

  it("renders measured points", () => {
    const { container } = render(
      <GrayscaleTracker points={mockPoints} targetGamma={2.4} />
    );
    const circles = container.querySelectorAll("circle");
    expect(circles.length).toBe(mockPoints.length);
  });
});
