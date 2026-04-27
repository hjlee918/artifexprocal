import { rgbToCss } from "../../lib/colorMath";

export interface GrayscalePoint {
  level: number;
  r: number;
  g: number;
  b: number;
  y: number;
  de: number;
  x: number;
  y_chromaticity: number;
}

export interface GrayscaleTrackerProps {
  points: GrayscalePoint[];
  targetGamma: number;
  width?: number;
  height?: number;
}

export function GrayscaleTracker({
  points,
  targetGamma,
  width = 500,
  height = 360,
}: GrayscaleTrackerProps) {
  const padding = { top: 10, right: 10, bottom: 40, left: 50 };
  const chartW = width - padding.left - padding.right;
  const chartH = height - padding.top - padding.bottom;

  const maxY = Math.max(100, ...points.map((p) => p.y));
  const maxDe = Math.max(5, ...points.map((p) => p.de));

  const xScale = (level: number) => (level / 100) * chartW;
  const yScale = (y: number) => chartH - (y / maxY) * chartH;
  const deScale = (de: number) => chartH - (de / maxDe) * chartH;

  const targetPoints = Array.from({ length: 101 }, (_, i) => {
    const normalized = i / 100;
    const y = Math.pow(normalized, targetGamma) * 100;
    return { level: i, y };
  });

  const gammaPath = targetPoints
    .map((p, i) => `${i === 0 ? "M" : "L"} ${padding.left + xScale(p.level)} ${padding.top + yScale(p.y)}`)
    .join(" ");

  const measuredGammaPath = points
    .map((p, i) => `${i === 0 ? "M" : "L"} ${padding.left + xScale(p.level)} ${padding.top + yScale(p.y)}`)
    .join(" ");

  const barW = points.length > 0 ? (chartW / points.length) * 0.6 : 0;

  return (
    <svg width={width} height={height} data-testid="grayscale-tracker">
      <rect x={0} y={0} width={width} height={height} fill="#1a1a1a" rx={4} />

      {[0, 25, 50, 75, 100].map((y) => (
        <line
          key={`y-${y}`}
          x1={padding.left}
          y1={padding.top + yScale(y)}
          x2={padding.left + chartW}
          y2={padding.top + yScale(y)}
          stroke="#333"
          strokeWidth={0.5}
        />
      ))}

      <path d={gammaPath} fill="none" stroke="#555" strokeWidth={1.5} strokeDasharray="4,4" />
      <path d={measuredGammaPath} fill="none" stroke="#2563eb" strokeWidth={2} />

      {points.map((p) => (
        <circle
          key={`pt-${p.level}`}
          cx={padding.left + xScale(p.level)}
          cy={padding.top + yScale(p.y)}
          r={3}
          fill="#2563eb"
        />
      ))}

      {points.map((p, i) => {
        const x = padding.left + xScale(p.level) - barW / 2;
        const barH = chartH - deScale(p.de);
        const color = p.de < 1 ? "#22c55e" : p.de < 3 ? "#f59e0b" : "#ef4444";
        return (
          <rect
            key={`de-${i}`}
            x={x}
            y={padding.top + deScale(p.de)}
            width={barW}
            height={barH}
            fill={color}
            opacity={0.5}
            rx={1}
          />
        );
      })}

      {points.map((p, i) => {
        const x = padding.left + xScale(p.level) - barW / 2;
        const baseY = height - 30;
        const barH = 24;
        return (
          <g key={`rgb-${i}`}>
            <rect x={x} y={baseY} width={barW / 3} height={barH} fill={rgbToCss(p.r, 0, 0)} rx={1} />
            <rect x={x + barW / 3} y={baseY} width={barW / 3} height={barH} fill={rgbToCss(0, p.g, 0)} rx={1} />
            <rect x={x + (2 * barW) / 3} y={baseY} width={barW / 3} height={barH} fill={rgbToCss(0, 0, p.b)} rx={1} />
          </g>
        );
      })}

      <text x={padding.left + chartW / 2} y={height - 2} textAnchor="middle" fill="#888" fontSize={10}>
        Patch Level (%)
      </text>
      <text
        x={12}
        y={padding.top + chartH / 2}
        textAnchor="middle"
        fill="#888"
        fontSize={10}
        transform={`rotate(-90, 12, ${padding.top + chartH / 2})`}
      >
        Y (nits)
      </text>
    </svg>
  );
}
