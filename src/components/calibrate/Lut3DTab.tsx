import { ThreeCanvas } from "../visualizations/ThreeCanvas";
import { LutCubeScene } from "../visualizations/LutCubeScene";

export interface Lut3DTabProps {
  lutSize?: number;
  has3DLut: boolean;
}

export function Lut3DTab({ lutSize, has3DLut }: Lut3DTabProps) {
  if (!has3DLut) {
    return (
      <div className="text-center py-12 text-gray-400">
        3D LUT was not generated for this session.
        <br />
        Select "Grayscale + 3D LUT" or "Full 3D LUT" tier for volumetric correction.
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-3 gap-4">
        <SummaryCard label="LUT Size" value={`${lutSize ?? 33}³`} />
        <SummaryCard label="Interpolation" value="Tetrahedral" />
        <SummaryCard label="Format" value=".cube / .3dl" />
      </div>

      <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
        <div className="text-xs text-gray-500 uppercase mb-2">3D LUT Cube</div>
        <div className="h-64">
          <ThreeCanvas>
            <LutCubeScene />
          </ThreeCanvas>
        </div>
      </div>
    </div>
  );
}

function SummaryCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-gray-800 border border-gray-800 rounded-lg p-3">
      <div className="text-xs text-gray-500 uppercase">{label}</div>
      <div className="text-xl font-semibold text-white">{value}</div>
    </div>
  );
}
