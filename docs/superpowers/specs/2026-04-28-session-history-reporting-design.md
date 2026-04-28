# Phase 7a — Session History, Detail View, and Comparison Design

**Date:** 2026-04-28
**Project:** ArtifexProCal
**Scope:** Backend query layer, frontend history table, session detail viewer, before/after comparison, raw data export (CSV/JSON)

---

## Goal

Enable users to browse past calibration sessions, inspect full measurement data and visualizations for any session, compare two sessions side-by-side, and export raw data for external analysis. All data is sourced from the existing SQLite persistence layer — no new storage schema is required beyond minor column additions.

---

## Architecture

### Backend (Rust)

| File | Responsibility |
|---|---|
| `crates/calibration-storage/src/query.rs` | Query layer: list, filter, paginate sessions; fetch full session detail; fetch readings and results |
| `crates/calibration-storage/src/export.rs` | Data export: `SessionExporter` with CSV and JSON writers |
| `src-tauri/src/ipc/models.rs` | DTOs: `SessionSummaryDto`, `SessionDetailDto`, `SessionFilterDto`, `ComparisonDto` |
| `src-tauri/src/ipc/commands.rs` | Commands: `list_sessions`, `get_session_detail`, `export_session_data` |
| `src-tauri/src/bindings_export.rs` | Add commands to specta exports |

### Frontend (React + TypeScript)

| File | Responsibility |
|---|---|
| `src/components/history/SessionTable.tsx` | Sortable, filterable, paginated session list |
| `src/components/history/SessionDetailView.tsx` | Full session viewer with charts, tables, summary cards |
| `src/components/history/SessionCompareView.tsx` | Side-by-side comparison of two sessions |
| `src/components/history/ExportMenu.tsx` | Export format dropdown (CSV / JSON) |
| `src/components/views/HistoryView.tsx` | Route entry point — replaces placeholder |

---

## Data Flow

```
User navigates to /history
  → list_sessions(filters, page) → SQLite query → SessionTable
User clicks a session row
  → get_session_detail(id) → SQLite query → SessionDetailView
User clicks "Compare" in detail view
  → Route to /history/compare/:sessionA/:sessionB
  → get_session_detail for both → SessionCompareView
User clicks "Export"
  → export_session_data(id, format) → SQLite query → file write → Tauri dialog
```

---

## Backend Design

### Query Layer (`calibration-storage/src/query.rs`)

```rust
pub struct SessionFilter {
    pub target_space: Option<String>,
    pub state: Option<String>,
    pub date_from: Option<i64>,
    pub date_to: Option<i64>,
    pub search: Option<String>,
}

pub struct SessionSummary {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    pub ended_at: Option<i64>,
    pub state: String,
    pub target_space: String,
    pub tier: String,
    pub patch_count: usize,
    pub gamma: Option<f64>,
    pub max_de: Option<f64>,
    pub avg_de: Option<f64>,
}

pub struct SessionDetail {
    pub summary: SessionSummary,
    pub config: SessionConfig,
    pub readings: Vec<PatchReading>,
    pub results: Option<ComputedResults>,
}

pub struct SessionQuery<'a> {
    conn: &'a Connection,
}

impl<'a> SessionQuery<'a> {
    pub fn new(conn: &'a Connection) -> Self;
    pub fn list(
        &self,
        filter: &SessionFilter,
        page: usize,
        per_page: usize,
    ) -> Result<(Vec<SessionSummary>, usize)>; // (items, total_count)
    pub fn get_detail(&self, session_id: &str) -> Result<Option<SessionDetail>>;
}
```

### Export Layer (`calibration-storage/src/export.rs`)

```rust
pub struct SessionExporter;

impl SessionExporter {
    pub fn export_csv(
        detail: &SessionDetail,
        writer: &mut dyn Write,
    ) -> std::io::Result<()>;

    pub fn export_json(
        detail: &SessionDetail,
        writer: &mut dyn Write,
    ) -> std::io::Result<()>;
}
```

**CSV format:**
```
patch_index, target_r, target_g, target_b, measured_x, measured_y, measured_z, de2000
0, 0.00, 0.00, 0.00, 0.52, 0.55, 0.62, 2.34
...
```

**JSON format:**
```json
{
  "session_id": "uuid",
  "name": "LG OLED SDR",
  "config": { ... },
  "results": { "gamma": 2.4, "max_de": 1.23, ... },
  "readings": [
    { "patch_index": 0, "target_rgb": [0,0,0], "measured_xyz": [0.52,0.55,0.62], "de2000": 2.34 }
  ]
}
```

### Schema Extensions

```sql
ALTER TABLE computed_results ADD COLUMN avg_de REAL;
ALTER TABLE computed_results ADD COLUMN lut_3d_size INTEGER;
ALTER TABLE computed_results ADD COLUMN lut_3d_json TEXT;
ALTER TABLE sessions ADD COLUMN ended_at INTEGER;
ALTER TABLE sessions ADD COLUMN tier TEXT;
```

Migration runs on app startup via `Storage::init_schema()` which already uses `CREATE TABLE IF NOT EXISTS`. We add a `migrate_v2()` method called after `init_schema()`.

---

## Frontend Design

### HistoryView Layout

```
┌─────────────────────────────────────────────────────┐
│  Session History                                    │
├─────────────────────────────────────────────────────┤
│  [Search _____]  [Target ▼]  [State ▼]  [Date ▼]   │
├─────────────────────────────────────────────────────┤
│  Date      Name        Target   Tier   Gamma  MaxΔE │
│  ───────────────────────────────────────────────────  │
│  2026-04-27  LG OLED SDR  Rec.709  Full3D  2.40  0.87│  [View] [Compare] [Export ▼]
│  2026-04-25  Sony HDR    Rec.2020 Grayscale 2.38  1.12│  [View] [Compare] [Export ▼]
│  ...                                                │
├─────────────────────────────────────────────────────┤
│  < Prev  Page 1 of 3  Next >                        │
└─────────────────────────────────────────────────────┘
```

### SessionDetailView Layout

```
┌─────────────────────────────────────────────────────┐
│  ← Back to History    LG OLED SDR    [Export ▼]    │
├─────────────────────────────────────────────────────┤
│  [Name]  [Date]  [Target]  [Tier]  [Patches]       │
│  [Gamma] [MaxΔE] [AvgΔE]   [WB R/G/B]              │
├─────────────────────────────────────────────────────┤
│  CIE 1931 Diagram            Grayscale Tracker      │
├─────────────────────────────────────────────────────┤
│  Patch Data Table (paginated)                        │
├─────────────────────────────────────────────────────┤
│  [Re-run Calibration]  [Compare with Another]       │
└─────────────────────────────────────────────────────┘
```

### SessionCompareView Layout

```
┌─────────────────────────────────────────────────────┐
│  Session A: [Dropdown]     Session B: [Dropdown]    │
├─────────────────────────────────────────────────────┤
│  Metric          Before (A)    After (B)    Delta   │
│  ───────────────────────────────────────────────────  │
│  Gamma           2.42          2.40         -0.02 ✓ │
│  Max ΔE2000      2.34          0.87         -1.47 ✓ │
│  Avg ΔE2000      1.12          0.45         -0.67 ✓ │
│  WB Error R      1.5%          0.3%         -1.2 ✓  │
├─────────────────────────────────────────────────────┤
│  [CIE A]                     [CIE B]                │
│  [Tracker A]                 [Tracker B]            │
├─────────────────────────────────────────────────────┤
│  Patch Delta Table                                   │
└─────────────────────────────────────────────────────┘
```

### Delta Color Logic
- **Green (✓):** Value improved (dE decreased, gamma closer to target, WB error decreased)
- **Red (✗):** Value worsened
- **Neutral (—):** No meaningful change

---

## TypeScript Bindings (Tauri IPC)

```typescript
// Models
interface SessionSummaryDto {
  id: string;
  name: string;
  created_at: number;
  ended_at: number | null;
  state: string;
  target_space: string;
  tier: string;
  patch_count: number;
  gamma: number | null;
  max_de: number | null;
  avg_de: number | null;
}

interface SessionDetailDto {
  summary: SessionSummaryDto;
  config: SessionConfigDto;
  readings: PatchReading[];
  results: ComputedResultsDto | null;
}

interface ComputedResultsDto {
  gamma: number;
  max_de: number;
  avg_de: number;
  white_balance_errors: [number, number, number];
  lut_1d_size: number;
  lut_3d_size: number | null;
}

// Commands
function listSessions(filter: SessionFilterDto, page: number, perPage: number): Promise<{ items: SessionSummaryDto[]; total: number }>;
function getSessionDetail(sessionId: string): Promise<SessionDetailDto>;
function exportSessionData(sessionId: string, format: "csv" | "json"): Promise<string>; // returns file path
```

---

## Error Handling

| Scenario | Backend | Frontend |
|---|---|---|
| Session not found | Return `None` | Show "Session not found" toast, redirect to /history |
| No readings for session | Return empty readings array | Show "No measurement data" placeholder |
| Export write failure | Return `Err(std::io::Error)` | Show error toast with path |
| Invalid filter params | Return empty list | No-op (table shows "No results") |
| Compare same session | N/A | Disable second dropdown, show "Select a different session" |

---

## Testing Strategy

### Backend Tests
- `query_test.rs`: Insert sessions with varying states/spaces, verify list/filter/pagination, verify get_detail roundtrip
- `export_test.rs`: Export known session to CSV, parse back and verify row count/values. Export to JSON, parse and verify structure.

### Frontend Tests
- `SessionTable.test.tsx`: Render with mock data, test sort click, test filter change, test pagination click
- `SessionDetailView.test.tsx`: Render with mock session, verify summary cards, verify chart components mount
- `SessionCompareView.test.tsx`: Render with two mock sessions, verify delta calculations, verify green/red coloring

### Integration Tests
- `history_integration_test.rs`: Full flow: run calibration → list_sessions → get_detail → export_csv → verify file contents

---

## Performance Considerations

- **Pagination:** `list_sessions` uses `LIMIT`/`OFFSET` in SQLite. Default 10 per page.
- **Readings load:** `get_session_detail` loads all readings for a session. For a Full3D session with 600+ patches × 5 reads = 3000 rows, this is still well within SQLite's performance envelope (<50ms). If needed later, paginate readings.
- **Export streaming:** CSV and JSON writers stream directly to file — no in-memory accumulation of large strings.

---

## Scope Exclusions (Phase 7b / 7c)

- **PDF report generation** — requires `reporting` crate (Phase 7b or 8)
- **HTML report generation** — same, requires templating engine
- **Session deletion** — UI only; backend already supports `ON DELETE CASCADE`
- **Session editing / renaming** — out of scope
- **Import external session data** — out of scope

---

## File Structure

| Create | Modify |
|---|---|
| `crates/calibration-storage/src/query.rs` | `crates/calibration-storage/src/schema.rs` (migration) |
| `crates/calibration-storage/src/export.rs` | `crates/calibration-storage/src/lib.rs` (exports) |
| `crates/calibration-storage/tests/query_test.rs` | `src-tauri/src/ipc/models.rs` (new DTOs) |
| `crates/calibration-storage/tests/export_test.rs` | `src-tauri/src/ipc/commands.rs` (new commands) |
| `src/components/history/SessionTable.tsx` | `src-tauri/src/bindings_export.rs` (exports) |
| `src/components/history/SessionDetailView.tsx` | `src/components/views/HistoryView.tsx` (replace placeholder) |
| `src/components/history/SessionCompareView.tsx` | |
| `src/components/history/ExportMenu.tsx` | |
| `src/components/__tests__/SessionTable.test.tsx` | |
| `src/components/__tests__/SessionDetailView.test.tsx` | |
| `src/components/__tests__/SessionCompareView.test.tsx` | |
| `crates/calibration-engine/tests/history_integration_test.rs` | |

---

## Spec Self-Review

**Placeholder scan:** No "TBD", "TODO", or incomplete sections. All code snippets show complete types and signatures. All test names are explicit.

**Internal consistency:** `SessionDetailDto` references `SessionConfigDto` which already exists in the bindings. `PatchReading` matches the existing frontend type. `ComputedResultsDto` fields match the `computed_results` table columns.

**Scope check:** This is a single focused phase — history browsing, detail viewing, comparison, and raw export. PDF/HTML reports are explicitly excluded.

**Ambiguity check:**
- "Delta color logic" is defined precisely (improved = green, worsened = red).
- "Export format" is strictly CSV or JSON.
- "Pagination" defaults to 10 per page.
- "Migration" runs automatically on startup.

All clear. No ambiguity found.
