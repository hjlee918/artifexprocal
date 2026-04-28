import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { Lut3DTab } from "../calibrate/Lut3DTab";

vi.mock("../visualizations/ThreeCanvas", () => ({
  ThreeCanvas: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="three-canvas">{children}</div>
  ),
}));

vi.mock("../visualizations/LutCubeScene", () => ({
  LutCubeScene: () => <div data-testid="lut-cube-scene" />,
}));

describe("Lut3DTab", () => {
  it("shows placeholder when no 3D LUT is available", () => {
    render(<Lut3DTab has3DLut={false} />);
    expect(screen.getByText(/3D LUT was not generated for this session/i)).toBeInTheDocument();
    expect(screen.getByText(/Select "Grayscale \+ 3D LUT" or "Full 3D LUT" tier/i)).toBeInTheDocument();
  });

  it("renders summary cards when 3D LUT is available", () => {
    render(<Lut3DTab has3DLut={true} lutSize={33} />);
    expect(screen.getByText("LUT Size")).toBeInTheDocument();
    expect(screen.getByText("33³")).toBeInTheDocument();
    expect(screen.getByText("Interpolation")).toBeInTheDocument();
    expect(screen.getByText("Tetrahedral")).toBeInTheDocument();
    expect(screen.getByText("Format")).toBeInTheDocument();
    expect(screen.getByText(".cube / .3dl")).toBeInTheDocument();
  });

  it("renders the 3D cube scene", () => {
    render(<Lut3DTab has3DLut={true} lutSize={33} />);
    expect(screen.getByTestId("three-canvas")).toBeInTheDocument();
    expect(screen.getByTestId("lut-cube-scene")).toBeInTheDocument();
  });

  it("defaults to 33³ when lutSize is not provided", () => {
    render(<Lut3DTab has3DLut={true} />);
    expect(screen.getByText("33³")).toBeInTheDocument();
  });
});
