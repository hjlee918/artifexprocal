interface GammaPoint {
  level: number; // 0-100
  y: number;
}

export function LiveGammaChart({
  targetGamma,
  measuredPoints,
  width = 400,
  height = 200,
}: {
  targetGamma: number;
  measuredPoints: GammaPoint[];
  width?: number;
  height?: number;
}) {
  const padding = { top: 10, right: 10, bottom: 30, left: 40 };
  const chartW = width - padding.left - padding.right;
  const chartH = height - padding.top - padding.bottom;

  const xScale = (level: number) => (level / 100) * chartW;
  const yScale = (y: number) => chartH - (y / 120) * chartH;

  const targetPoints = Array.from({ length: 101 }, (_, i) => {
    const level = i;
    const normalized = level / 100;
    const y = Math.pow(normalized, targetGamma) * 100;
    return { level, y };
  });

  const targetPath = targetPoints
    .map((p, i) => `${i === 0 ? "M" : "L"} ${padding.left + xScale(p.level)} ${padding.top + yScale(p.y)}`)
    .join(" ");

  const measuredPath = measuredPoints
    .map((p, i) => `${i === 0 ? "M" : "L"} ${padding.left + xScale(p.level)} ${padding.top + yScale(p.y)}`)
    .join(" ");

  return (
    <svg width={width} height={height}>
      {/* Grid lines */}
      {[0, 25, 50, 75, 100].map((y) => (
        <line
          key={y}
          x1={padding.left}
          y1={padding.top + yScale(y)}
          x2={padding.left + chartW}
          y2={padding.top + yScale(y)}
          stroke="#333"
          strokeWidth={0.5}
        />
      ))}

      {/* Target gamma curve (dashed) */}
      <path d={targetPath} fill="none" stroke="#555" strokeWidth={1.5} strokeDasharray="4,4" />

      {/* Measured points */}
      <path d={measuredPath} fill="none" stroke="#2563eb" strokeWidth={2} />
      {measuredPoints.map((p) => (
        <circle
          key={p.level}
          cx={padding.left + xScale(p.level)}
          cy={padding.top + yScale(p.y)}
          r={3}
          fill="#2563eb"
        />
      ))}

      {/* Axes */}
      <text x={padding.left + chartW / 2} y={height - 5} textAnchor="middle" fill="#888" fontSize={10}>
        Patch Level (%)
      </text>
      <text
        x={10}
        y={padding.top + chartH / 2}
        textAnchor="middle"
        fill="#888"
        fontSize={10}
        transform={`rotate(-90, 10, ${padding.top + chartH / 2})`}
      >
        Y (nits)
      </text>
    </svg>
  );
}
