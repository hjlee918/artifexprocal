import { useRef, useEffect } from "react";

export interface Gamut {
  red: [number, number];
  green: [number, number];
  blue: [number, number];
  white: [number, number];
}

export interface CIEDiagramProps {
  locus: [number, number][];
  targetGamut: Gamut;
  measuredGamut?: Gamut;
  size?: number;
  diagramType?: "xy" | "uv";
}

export function CIEDiagram({
  locus,
  targetGamut,
  measuredGamut,
  size = 400,
  diagramType = "xy",
}: CIEDiagramProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const padding = 40;
  const chartSize = size - padding * 2;

  const toCanvas = (u: number, v: number) => {
    const xRange = diagramType === "xy" ? [0.0, 0.8] : [0.0, 0.65];
    const yRange = diagramType === "xy" ? [0.0, 0.9] : [0.0, 0.65];
    const x = padding + ((u - xRange[0]) / (xRange[1] - xRange[0])) * chartSize;
    const y = padding + (1 - (v - yRange[0]) / (yRange[1] - yRange[0])) * chartSize;
    return [x, y] as const;
  };

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.clearRect(0, 0, size, size);

    ctx.fillStyle = "#1a1a1a";
    ctx.fillRect(0, 0, size, size);

    // Grid
    ctx.strokeStyle = "#333";
    ctx.lineWidth = 0.5;
    const xTicks = diagramType === "xy" ? [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7] : [0.1, 0.2, 0.3, 0.4, 0.5];
    const yTicks = diagramType === "xy" ? [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8] : [0.1, 0.2, 0.3, 0.4, 0.5];
    for (const t of xTicks) {
      const [x] = toCanvas(t, 0);
      ctx.beginPath();
      ctx.moveTo(x, padding);
      ctx.lineTo(x, size - padding);
      ctx.stroke();
      ctx.fillStyle = "#666";
      ctx.font = "10px sans-serif";
      ctx.fillText(t.toFixed(1), x - 6, size - padding + 14);
    }
    for (const t of yTicks) {
      const [, y] = toCanvas(0, t);
      ctx.beginPath();
      ctx.moveTo(padding, y);
      ctx.lineTo(size - padding, y);
      ctx.stroke();
      ctx.fillStyle = "#666";
      ctx.font = "10px sans-serif";
      ctx.fillText(t.toFixed(1), padding - 22, y + 3);
    }

    // Axes labels
    ctx.fillStyle = "#888";
    ctx.font = "12px sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(diagramType === "xy" ? "x" : "u'", size / 2, size - 4);
    ctx.save();
    ctx.translate(12, size / 2);
    ctx.rotate(-Math.PI / 2);
    ctx.fillText(diagramType === "xy" ? "y" : "v'", 0, 0);
    ctx.restore();

    // Spectral locus
    if (locus.length > 0) {
      ctx.strokeStyle = "#aaa";
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      const [x0, y0] = toCanvas(locus[0][0], locus[0][1]);
      ctx.moveTo(x0, y0);
      for (let i = 1; i < locus.length; i++) {
        const [x, y] = toCanvas(locus[i][0], locus[i][1]);
        ctx.lineTo(x, y);
      }
      ctx.stroke();
    }

    // Target gamut triangle
    ctx.strokeStyle = "#22c55e";
    ctx.lineWidth = 2;
    ctx.beginPath();
    const [rtx, rty] = toCanvas(targetGamut.red[0], targetGamut.red[1]);
    const [gtx, gty] = toCanvas(targetGamut.green[0], targetGamut.green[1]);
    const [btx, bty] = toCanvas(targetGamut.blue[0], targetGamut.blue[1]);
    ctx.moveTo(rtx, rty);
    ctx.lineTo(gtx, gty);
    ctx.lineTo(btx, bty);
    ctx.closePath();
    ctx.stroke();

    // Target white point
    ctx.fillStyle = "#22c55e";
    ctx.beginPath();
    const [twx, twy] = toCanvas(targetGamut.white[0], targetGamut.white[1]);
    ctx.arc(twx, twy, 4, 0, Math.PI * 2);
    ctx.fill();

    // Measured gamut triangle
    if (measuredGamut) {
      ctx.strokeStyle = "#ef4444";
      ctx.lineWidth = 2;
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      const [rmx, rmy] = toCanvas(measuredGamut.red[0], measuredGamut.red[1]);
      const [gmx, gmy] = toCanvas(measuredGamut.green[0], measuredGamut.green[1]);
      const [bmx, bmy] = toCanvas(measuredGamut.blue[0], measuredGamut.blue[1]);
      ctx.moveTo(rmx, rmy);
      ctx.lineTo(gmx, gmy);
      ctx.lineTo(bmx, bmy);
      ctx.closePath();
      ctx.stroke();
      ctx.setLineDash([]);

      // Measured white point
      ctx.fillStyle = "#ef4444";
      ctx.beginPath();
      const [mwx, mwy] = toCanvas(measuredGamut.white[0], measuredGamut.white[1]);
      ctx.arc(mwx, mwy, 4, 0, Math.PI * 2);
      ctx.fill();
    }
  }, [locus, targetGamut, measuredGamut, size, diagramType]);

  return (
    <canvas
      ref={canvasRef}
      width={size}
      height={size}
      className="rounded-lg border border-gray-700 bg-[#1a1a1a]"
      data-testid="cie-diagram"
    />
  );
}
