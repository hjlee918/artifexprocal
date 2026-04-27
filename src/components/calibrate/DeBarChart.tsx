interface DePoint {
  level: number;
  de: number;
}

export function DeBarChart({
  points,
  width = 500,
  height = 200,
}: {
  points: DePoint[];
  width?: number;
  height?: number;
}) {
  const padding = { top: 10, right: 10, bottom: 30, left: 40 };
  const chartW = width - padding.left - padding.right;
  const chartH = height - padding.top - padding.bottom;
  const maxDe = Math.max(5, ...points.map((p) => p.de));

  const barWidth = chartW / points.length * 0.7;
  const barSpacing = chartW / points.length;

  return (
    <svg width={width} height={height}>
      {/* Grid lines */}
      {[1, 3, 5].map((y) => (
        <line
          key={y}
          x1={padding.left}
          y1={padding.top + chartH - (y / maxDe) * chartH}
          x2={padding.left + chartW}
          y2={padding.top + chartH - (y / maxDe) * chartH}
          stroke={y === 1 ? "#22c55e22" : y === 3 ? "#f59e0b22" : "#ef444422"}
          strokeWidth={0.5}
        />
      ))}

      {/* Bars */}
      {points.map((p, i) => {
        const barH = (p.de / maxDe) * chartH;
        const x = padding.left + i * barSpacing + (barSpacing - barWidth) / 2;
        const y = padding.top + chartH - barH;
        const color = p.de < 1 ? "#22c55e" : p.de < 3 ? "#f59e0b" : "#ef4444";

        return (
          <rect key={i} x={x} y={y} width={barWidth} height={barH} fill={color} rx={2} />
        );
      })}

      {/* Threshold labels */}
      <text x={padding.left + chartW - 5} y={padding.top + chartH - (1 / maxDe) * chartH - 3} textAnchor="end" fill="#22c55e" fontSize={9}>
        dE = 1
      </text>
      <text x={padding.left + chartW - 5} y={padding.top + chartH - (3 / maxDe) * chartH - 3} textAnchor="end" fill="#f59e0b" fontSize={9}>
        dE = 3
      </text>

      <text x={padding.left + chartW / 2} y={height - 5} textAnchor="middle" fill="#888" fontSize={10}>
        Patch Level (%)
      </text>
    </svg>
  );
}
