/**
 * Color math utilities for visualization components.
 * All formulae from CIE standards and authoritative color-science references.
 */

import type { Chromaticity, GamutDto } from "../bindings";

/** Convert CIE XYZ to CIE 1931 xy chromaticity coordinates. */
export function xyzToXy(x: number, y: number, z: number): Chromaticity {
  const sum = x + y + z;
  if (sum === 0) return { x: 0, y: 0 };
  return { x: x / sum, y: y / sum };
}

/** Convert CIE XYZ to CIE 1976 u'v' chromaticity coordinates. */
export function xyzToUv(x: number, y: number, z: number): Chromaticity {
  const denom = -2 * x + 12 * y + 3 * z;
  if (denom === 0) return { x: 0, y: 0 };
  return { x: (4 * x) / denom, y: (9 * y) / denom };
}

/** McCamy's CCT approximation from CIE 1931 xy. */
export function approximateCct(x: number, y: number): number {
  const n = (x - 0.332) / (0.1858 - y);
  return -449 * n ** 3 + 3525 * n ** 2 - 6823.3 * n + 5520.33;
}

/** Return standard primary/white chromaticities for a named target space. */
export function getTargetGamut(targetSpace: string): GamutDto {
  const c = (x: number, y: number): Chromaticity => ({ x, y });
  switch (targetSpace) {
    case "Rec.709":
    case "sRGB":
      return {
        red: c(0.64, 0.33),
        green: c(0.3, 0.6),
        blue: c(0.15, 0.06),
        white: c(0.3127, 0.329),
      };
    case "Rec.2020":
      return {
        red: c(0.708, 0.292),
        green: c(0.17, 0.797),
        blue: c(0.131, 0.046),
        white: c(0.3127, 0.329),
      };
    case "DCI-P3":
      return {
        red: c(0.68, 0.32),
        green: c(0.265, 0.69),
        blue: c(0.15, 0.06),
        white: c(0.314, 0.351),
      };
    case "Adobe RGB":
      return {
        red: c(0.64, 0.33),
        green: c(0.21, 0.71),
        blue: c(0.15, 0.06),
        white: c(0.3127, 0.329),
      };
    default:
      throw new Error(`Unknown target space: ${targetSpace}`);
  }
}

/** Clamp 0-1 and convert to 8-bit sRGB CSS string. */
export function rgbToCss(r: number, g: number, b: number): string {
  const to8 = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255);
  return `rgb(${to8(r)} ${to8(g)} ${to8(b)})`;
}

export interface ColorCheckerPatch {
  name: string;
  srgb: [number, number, number];
}

/** Classic 24-patch ColorChecker sRGB values (D65, normalized 0-1). */
export const COLORCHECKER_PATCHES: ColorCheckerPatch[] = [
  { name: "Dark Skin", srgb: [0.4, 0.267, 0.2] },
  { name: "Light Skin", srgb: [0.604, 0.424, 0.337] },
  { name: "Blue Sky", srgb: [0.184, 0.294, 0.435] },
  { name: "Foliage", srgb: [0.329, 0.42, 0.247] },
  { name: "Blue Flower", srgb: [0.314, 0.282, 0.475] },
  { name: "Bluish Green", srgb: [0.239, 0.533, 0.537] },
  { name: "Orange", srgb: [0.729, 0.424, 0.196] },
  { name: "Purplish Blue", srgb: [0.259, 0.235, 0.537] },
  { name: "Moderate Red", srgb: [0.612, 0.306, 0.322] },
  { name: "Purple", srgb: [0.329, 0.227, 0.408] },
  { name: "Yellow Green", srgb: [0.541, 0.6, 0.216] },
  { name: "Orange Yellow", srgb: [0.714, 0.506, 0.204] },
  { name: "Blue", srgb: [0.145, 0.188, 0.463] },
  { name: "Green", srgb: [0.294, 0.475, 0.255] },
  { name: "Red", srgb: [0.6, 0.231, 0.212] },
  { name: "Yellow", srgb: [0.827, 0.69, 0.165] },
  { name: "Magenta", srgb: [0.475, 0.282, 0.475] },
  { name: "Cyan", srgb: [0.208, 0.486, 0.616] },
  { name: "White 9.5", srgb: [0.957, 0.957, 0.957] },
  { name: "Neutral 8", srgb: [0.784, 0.784, 0.784] },
  { name: "Neutral 6.5", srgb: [0.612, 0.612, 0.612] },
  { name: "Neutral 5", srgb: [0.431, 0.431, 0.431] },
  { name: "Neutral 3.5", srgb: [0.267, 0.267, 0.267] },
  { name: "Black 2", srgb: [0.114, 0.114, 0.114] },
];
