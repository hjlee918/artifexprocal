# Phase 7b — Report Generation Implementation Plan

**Date:** 2026-04-29
**Project:** ArtifexProCal
**Branch:** `phase7b-report-generation`

---

## Overview

Build a `reporting` crate that generates professional calibration reports from session data. Three templates (Quick Summary, Detailed, Pre/Post Comparison), two formats (HTML self-contained, PDF). Reports embed inline SVG charts (CIE diagram, grayscale tracker, dE bars) and CSS.

---

## Tasks

### Task 0: Create `reporting` crate shell
- `crates/reporting/Cargo.toml` with dependencies: `calibration-storage`, `calibration-core`, `color-science`, `thiserror`
- `crates/reporting/src/lib.rs` with crate root
- Add `reporting` to workspace `Cargo.toml`
- Verify `cargo check -p reporting` passes

### Task 1: Report types and error handling
- `src/types.rs`: `ReportTemplate`, `ReportFormat`, `ReportError` enums
- `src/lib.rs`: Re-export types
- Unit tests for enum serialization

### Task 2: SVG chart generation
- `src/svg.rs`: `cie_diagram_svg()`, `grayscale_tracker_svg()`, `de_bar_chart_svg()`
- Use existing spectral locus data from `color_science::diagrams`
- Generate simple SVG with paths, circles, text
- Tests: verify SVG string contains expected elements

### Task 3: HTML template engine
- `src/assets.rs`: Inline CSS constants
- `src/template.rs`: `render_html()` for all three templates
  - QuickSummary: 1-page, metrics cards, mini charts
  - Detailed: multi-page sections, full charts, dE table
  - PrePostComparison: side-by-side layout, delta values
- Tests: render each template with mock data, assert HTML contains session name and metrics

### Task 4: PDF generation
- `src/pdf.rs`: `html_to_pdf()` using `printpdf`
- Generate PDF from HTML string
- Tests: generate PDF, assert non-empty and valid PDF magic bytes

### Task 5: ReportEngine orchestration
- `src/engine.rs`: `ReportEngine::generate()` that composes template + format
- Handle `PrePostComparison` requiring two sessions
- Tests: integration test with mock session data

### Task 6: Tauri IPC integration
- `src-tauri/src/ipc/models.rs`: `ReportRequestDto`, `ReportResponseDto`, `ReportTemplate` specta derives
- `src-tauri/src/ipc/commands.rs`: `generate_report` command
- `src-tauri/src/bindings_export.rs`: Add to specta exports
- `src-tauri/Cargo.toml`: Add `reporting` dependency
- Backend tests: verify command accepts request and returns path

### Task 7: Regenerate TypeScript bindings
- Run `cargo test -p artifexprocal bindings_export::tests::export_typescript_bindings -- --nocapture`
- Verify `src/bindings.ts` includes `ReportTemplate`, `ReportRequestDto`, `ReportResponseDto`

### Task 8: Frontend — ReportDialog component
- `src/components/history/ReportDialog.tsx`
  - Template selector (Quick Summary / Detailed / Pre/Post Comparison)
  - Format selector (PDF / HTML)
  - Optional compare session dropdown
  - Preview button (opens HTML in new tab)
  - Download button
- `src/components/__tests__/ReportDialog.test.tsx`
  - Render, select template, verify command called with correct params

### Task 9: Frontend — Wire into ExportMenu and HistoryView
- `src/components/history/ExportMenu.tsx`: Add "Generate Report..." option
- `src/components/views/HistoryView.tsx`: Wire ReportDialog open/close state
- Update `SessionDetailView` to show "Generate Report" button

### Task 10: Full test suite run
- `cargo test --workspace` — all 200+ tests pass
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- `npx vitest run` — all frontend tests pass
- Commit and push `phase7b-report-generation` branch

---

## Dependencies to Add

```toml
# crates/reporting/Cargo.toml
[dependencies]
calibration-storage = { path = "../calibration-storage" }
calibration-core = { path = "../calibration-core" }
color-science = { path = "../color-science" }
thiserror = "1.0"

# For PDF generation (Task 4)
printpdf = "0.7"

# For HTML-to-PDF rasterization (alternative if printpdf is insufficient)
# headless_chrome = "1.0"  # Optional, heavier dependency
```

```toml
# src-tauri/Cargo.toml
[dependencies]
reporting = { path = "../crates/reporting" }
```

---

## Risk Mitigation

| Risk | Mitigation |
|---|---|
| `printpdf` too limited for complex charts | Fallback to self-contained HTML + browser print for PDF |
| SVG rendering in PDF | Rasterize SVGs to PNG before embedding, or use a library that supports SVG in PDF |
| Large HTML files for detailed reports | Streaming string building; no DOM manipulation |
| Report generation blocking UI | Async Tauri command with progress events (optional for this phase) |

---

## File Structure (Final)

```
crates/reporting/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── types.rs
│   ├── svg.rs
│   ├── template.rs
│   ├── assets.rs
│   ├── pdf.rs
│   └── engine.rs
└── tests/
    ├── types_test.rs
    ├── svg_test.rs
    ├── template_test.rs
    ├── pdf_test.rs
    └── integration_test.rs

src/components/history/
├── ReportDialog.tsx
├── ExportMenu.tsx (modified)
└── __tests__/
    └── ReportDialog.test.tsx
```

---

## Estimates

| Task | Est. Time | Risk |
|---|---|---|
| Task 0: Crate shell | 5 min | Low |
| Task 1: Types + errors | 10 min | Low |
| Task 2: SVG charts | 45 min | Medium (SVG math) |
| Task 3: HTML templates | 60 min | Medium (CSS/layout) |
| Task 4: PDF generation | 30 min | Medium (printpdf API) |
| Task 5: ReportEngine | 20 min | Low |
| Task 6: IPC integration | 30 min | Low |
| Task 7: Bindings regen | 10 min | Low |
| Task 8: ReportDialog | 45 min | Medium (UI state) |
| Task 9: Wire into views | 20 min | Low |
| Task 10: Full test suite | 30 min | Low |
| **Total** | **~5.5 hrs** | |

---

## Acceptance Criteria

- [ ] All three templates render valid HTML with correct data
- [ ] PDF generation produces valid PDF files (>1KB, correct magic bytes)
- [ ] Frontend can generate reports from SessionDetailView
- [ ] Pre/Post comparison requires two sessions and shows deltas
- [ ] 197+ Rust tests pass, clippy clean
- [ ] 8+ frontend tests pass
- [ ] Branch pushed to origin
