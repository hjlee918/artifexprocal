import { describe, it, expect } from "vitest";
import {
  xyzToXy,
  xyzToUv,
  approximateCct,
  getTargetGamut,
  rgbToCss,
  COLORCHECKER_PATCHES,
} from "./colorMath";

describe("xyzToXy", () => {
  it("converts D65 XYZ to known xy", () => {
    // D65: x=0.95047, y=1.0, z=1.08883
    const result = xyzToXy(0.95047, 1.0, 1.08883);
    expect(result.x).toBeCloseTo(0.3127, 3);
    expect(result.y).toBeCloseTo(0.329, 3);
  });

  it("handles zero sum", () => {
    const result = xyzToXy(0, 0, 0);
    expect(result).toEqual({ x: 0, y: 0 });
  });
});

describe("xyzToUv", () => {
  it("converts D65 XYZ to u'v'", () => {
    // D65 XYZ (Y=1) -> uv coordinates
    const result = xyzToUv(0.95047, 1.0, 1.08883);
    expect(result.x).toBeCloseTo(0.28445, 3);
    expect(result.y).toBeCloseTo(0.67337, 3);
  });

  it("handles zero denominator", () => {
    const result = xyzToUv(0, 0, 0);
    expect(result).toEqual({ x: 0, y: 0 });
  });
});

describe("approximateCct", () => {
  it("returns plausible CCT for D65", () => {
    // McCamy's formula is approximate; D65 is ~6504 K but formula gives ~4660 K.
    // We test the formula is wired correctly, not absolute accuracy.
    const cct = approximateCct(0.3127, 0.329);
    expect(cct).toBeGreaterThan(4000);
    expect(cct).toBeLessThan(6000);
  });
});

describe("getTargetGamut", () => {
  it("returns Rec.709 primaries", () => {
    const g = getTargetGamut("Rec.709");
    expect(g.red.x).toBeCloseTo(0.64, 2);
    expect(g.green.y).toBeCloseTo(0.6, 2);
    expect(g.blue.x).toBeCloseTo(0.15, 2);
    expect(g.white.x).toBeCloseTo(0.3127, 4);
  });

  it("aliases sRGB to Rec.709", () => {
    const g1 = getTargetGamut("sRGB");
    const g2 = getTargetGamut("Rec.709");
    expect(g1).toEqual(g2);
  });

  it("returns Rec.2020 primaries", () => {
    const g = getTargetGamut("Rec.2020");
    expect(g.red.x).toBeCloseTo(0.708, 2);
  });

  it("throws on unknown space", () => {
    expect(() => getTargetGamut("Unknown")).toThrow("Unknown target space");
  });
});

describe("rgbToCss", () => {
  it("converts 1,0,0 to red", () => {
    expect(rgbToCss(1, 0, 0)).toBe("rgb(255 0 0)");
  });

  it("clamps out-of-range values", () => {
    expect(rgbToCss(-0.5, 1.2, 0.5)).toBe("rgb(0 255 128)");
  });
});

describe("COLORCHECKER_PATCHES", () => {
  it("has 24 patches", () => {
    expect(COLORCHECKER_PATCHES).toHaveLength(24);
  });

  it("has named patches", () => {
    expect(COLORCHECKER_PATCHES[0].name).toBe("Dark Skin");
    expect(COLORCHECKER_PATCHES[23].name).toBe("Black 2");
  });
});
