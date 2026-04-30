pub const REPORT_CSS: &str = r#":root {
  --primary: #3b82f6;
  --text: #1a1a1a;
  --muted: #666666;
  --bg: #ffffff;
  --card-bg: #f8f9fa;
  --border: #e5e5e5;
  --success: #22c55e;
  --danger: #ef4444;
}

* { box-sizing: border-box; }

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  color: var(--text);
  background: var(--bg);
  margin: 0;
  padding: 40px;
  line-height: 1.5;
}

.header {
  border-bottom: 2px solid var(--primary);
  padding-bottom: 16px;
  margin-bottom: 24px;
}

.header h1 {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 4px 0;
  color: var(--text);
}

.header .subtitle {
  font-size: 14px;
  color: var(--muted);
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.metric-card {
  background: var(--card-bg);
  border-radius: 8px;
  padding: 16px;
  text-align: center;
}

.metric-label {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--muted);
  margin-bottom: 4px;
}

.metric-value {
  font-size: 28px;
  font-weight: 600;
  color: var(--primary);
}

.metric-unit {
  font-size: 12px;
  color: var(--muted);
}

.section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border);
}

table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 16px;
}

th, td {
  padding: 8px 12px;
  text-align: left;
  border-bottom: 1px solid var(--border);
}

th {
  font-weight: 600;
  font-size: 12px;
  text-transform: uppercase;
  color: var(--muted);
  background: var(--card-bg);
}

td {
  font-size: 13px;
}

.footer {
  margin-top: 40px;
  padding-top: 16px;
  border-top: 1px solid var(--border);
  font-size: 11px;
  color: var(--muted);
  text-align: center;
}

.comparison-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.comparison-card {
  background: var(--card-bg);
  border-radius: 8px;
  padding: 20px;
}

.comparison-card h3 {
  margin-top: 0;
  font-size: 16px;
}

.delta-positive { color: var(--success); }
.delta-negative { color: var(--danger); }

.chart-container {
  display: flex;
  justify-content: center;
  margin: 16px 0;
}

@media print {
  body { padding: 20px; }
  .metric-grid { grid-template-columns: repeat(3, 1fr); }
}
"#;
