import { COLORCHECKER_PATCHES, rgbToCss } from "../../lib/colorMath";

export interface ColorCheckerPatchData {
  measuredRgb: [number, number, number];
  de2000: number;
}

export interface ColorCheckerGridProps {
  patches: ColorCheckerPatchData[];
}

export function ColorCheckerGrid({ patches }: ColorCheckerGridProps) {
  const displayPatches = patches.slice(0, 24);

  const getDeColor = (de: number) => {
    if (de < 1) return "text-green-400";
    if (de < 3) return "text-yellow-400";
    return "text-red-400";
  };

  const avgDe =
    displayPatches.length > 0
      ? displayPatches.reduce((s, p) => s + p.de2000, 0) / displayPatches.length
      : 0;

  return (
    <div className="space-y-3" data-testid="colorchecker-grid">
      <div className="grid grid-cols-6 gap-1">
        {displayPatches.map((patch, i) => {
          const ref = COLORCHECKER_PATCHES[i];
          return (
            <div
              key={i}
              className="relative rounded overflow-hidden border border-gray-700"
              style={{ aspectRatio: "1" }}
            >
              <div
                className="absolute inset-0"
                style={{ backgroundColor: rgbToCss(...patch.measuredRgb) }}
              />
              <div
                className="absolute top-0 left-0 w-3 h-3 border-r border-b border-gray-500"
                style={{ backgroundColor: rgbToCss(...ref.srgb) }}
              />
              <div className="absolute inset-0 flex items-end justify-end p-1">
                <span className={`text-[10px] font-bold bg-black/60 px-1 rounded ${getDeColor(patch.de2000)}`}>
                  {patch.de2000.toFixed(1)}
                </span>
              </div>
            </div>
          );
        })}
      </div>
      <div className="flex justify-between text-xs text-gray-500">
        <span>Small corner = reference</span>
        <span>Avg dE: {avgDe.toFixed(2)}</span>
      </div>
    </div>
  );
}
