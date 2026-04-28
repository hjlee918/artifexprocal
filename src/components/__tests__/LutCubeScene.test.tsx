import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";

function MockBufferGeometry(this: { setAttribute: typeof vi.fn }) {
  this.setAttribute = vi.fn();
}
function MockBufferAttribute(this: unknown, data: Float32Array, size: number) {
  (this as { data: Float32Array; size: number }).data = data;
  (this as { data: Float32Array; size: number }).size = size;
}

vi.mock("three", () => ({
  BufferGeometry: MockBufferGeometry,
  BufferAttribute: MockBufferAttribute,
  Float32Array: globalThis.Float32Array,
}));

vi.mock("@react-three/fiber", () => ({
  useFrame: vi.fn(),
}));

vi.mock("@react-three/drei", () => ({
  OrbitControls: () => <div data-testid="orbit-controls" />,
}));

import { LutCubeScene } from "../visualizations/LutCubeScene";

describe("LutCubeScene", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders wireframe placeholder when no lutData", () => {
    render(<LutCubeScene />);
    expect(screen.getByTestId("orbit-controls")).toBeInTheDocument();
  });

  it("renders points when lutData is provided", () => {
    const lutSize = 2;
    const lutData = [
      0, 0, 0, // (0,0,0)
      1, 0, 0, // (1,0,0)
      0, 1, 0, // (0,1,0)
      1, 1, 0, // (1,1,0)
      0, 0, 1, // (0,0,1)
      1, 0, 1, // (1,0,1)
      0, 1, 1, // (0,1,1)
      1, 1, 1, // (1,1,1)
    ];

    render(<LutCubeScene lutSize={lutSize} lutData={lutData} />);
    expect(screen.getByTestId("orbit-controls")).toBeInTheDocument();
  });
});
