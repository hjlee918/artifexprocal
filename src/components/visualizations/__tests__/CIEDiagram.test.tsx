import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { CIEDiagram } from "../CIEDiagram";

const mockLocus: [number, number][] = [
  [0.174, 0.005],
  [0.173, 0.005],
  [0.171, 0.005],
];

const mockGamut = {
  red: [0.64, 0.33] as [number, number],
  green: [0.3, 0.6] as [number, number],
  blue: [0.15, 0.06] as [number, number],
  white: [0.3127, 0.329] as [number, number],
};

describe("CIEDiagram", () => {
  it("renders canvas", () => {
    const { getByTestId } = render(
      <CIEDiagram locus={mockLocus} targetGamut={mockGamut} />
    );
    expect(getByTestId("cie-diagram")).toBeInTheDocument();
    expect(getByTestId("cie-diagram").tagName).toBe("CANVAS");
  });

  it("renders with uv diagram type", () => {
    const { getByTestId } = render(
      <CIEDiagram locus={mockLocus} targetGamut={mockGamut} diagramType="uv" />
    );
    expect(getByTestId("cie-diagram")).toBeInTheDocument();
  });
});
