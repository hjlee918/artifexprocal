# module-meter

MeterModule — `CalibrationModule` implementation for colorimeters and spectrophotometers.

## Phase 1 Export Conventions

The JSON and CSV export formats include patch metadata fields (`patchRgb`, `patchBitDepth`, `patchColorspace`) that describe the RGB stimulus displayed by a pattern generator during measurement.

In Phase 1, there is no PatternModule active. This means every measurement is a standalone spot read with no known patch source. The export represents this state consistently:

- `patchColorspace`: `""` (empty string) — no colorspace declared
- `patchRgb`: `null` — no patch RGB values available
- `patchBitDepth`: `null` — no patch bit depth available

In CSV form, the four patch columns (`patch_r`, `patch_g`, `patch_b`, `patch_bit_depth`) are empty cells.

When PatternModule is introduced in Phase 4, these fields will be populated with real values and a non-empty `patchColorspace` (e.g., `"BT.709"`, `"BT.2020"`, `"DCI-P3"`). The schema enforces this correlation via JSON Schema `if/then/else`: empty colorspace requires null patch fields, and non-empty colorspace requires concrete object/integer patch fields.

## Phase 1 IPC Commands

The MeterModule exposes 16 commands via the `CalibrationModule::handle_command` interface:

| Command | Description |
|---------|-------------|
| `detect` | Enumerate connected instruments |
| `connect` | Open an instrument, return a meter ID |
| `disconnect` | Close an instrument |
| `read` | Single spot read |
| `read_continuous` | Begin streaming reads |
| `stop_continuous` | End streaming reads |
| `set_register` | Store a measurement into a register slot |
| `clear_register` | Clear a register slot |
| `get_all_registers` | Return all populated registers |
| `export_json` | Export measurement history as JSON (schema-validated) |
| `export_csv` | Export measurement history as CSV (RFC 4180, 34 columns) |
| `clear_history` | Clear measurement history |
| `list_active` | List currently connected meters with metadata |
| `probe` | Self-test instrument connectivity |
| `get_config` | Get current meter configuration |
| `set_config` | Set meter configuration (rejected during continuous read) |

## Running the Tests

No hardware or environment variables are required. All tests use `FakeMeter`.

```bash
# Run only this crate's integration tests (33 tests, ~0.2s)
cargo test -p module-meter

# Run the full workspace (72 tests across color-science, app-core, hal, hal-meters, module-meter)
cargo test --workspace
```

Expected output: all passing, no warnings.

## Phase 1 Stubs, Simplifications, and Deferrals

**Stub:** `FakeMeter` is the only `Meter` implementation. Real drivers (i1 Display Pro native HID, i1 Pro 2 native USB, ArgyllCMS PTY) are not yet integrated.

**Simplification:** Measurement history is an in-memory `VecDeque<MeasurementResult>` capped at 1000 entries (FIFO eviction). SQLite persistence is deferred to Phase 6+.

**Simplification:** `MeterConfig.averaging_count` and `integration_time_ms` are stored on `FakeMeter` but ignored by `read_xyz()`. Real drivers will act on these fields in Phase 2+.

**Simplification:** `probe()` is synchronous. See `docs/LESSONS_LEARNED.md` §13 for the async transition plan.

**Deferral:** `SettingsStore` wiring is absent — register persistence across app restarts is not implemented.

**Deferral:** Only three register slots exist (`Current`, `Reference`, `W`). The remaining seven (`K`, `R`, `G`, `B`, `C`, `M`, `Y`) arrive in Phase 2.

**Deferral:** `DisplayController`, `PatternGenerator`, `Lut1D`, `Lut3D`, and related types live in `crates/hal-future-traits/` as design sketches. They are not active dependencies.

## Phase 2 Driver Integration Guide

The integration point is `cmd_connect` in `src/lib.rs` (~line 111), where `FakeMeter::new()` is called. Replace this with driver enumeration and platform dispatch.

The only contract a real driver must satisfy is the `Meter` trait in `crates/hal/src/meter.rs`: `probe()`, `read_xyz()`, `set_mode()`, `set_config()`, `disconnect()`.

Blocking I/O (HID USB, ArgyllCMS PTY) must be wrapped in `tokio::task::spawn_blocking`. The planned location for these adapters is `src/driver_adapter.rs` (not yet implemented).

**Trap:** `probe()` is currently synchronous. When adding the first real driver, revisit `docs/LESSONS_LEARNED.md` §13 before deciding whether to make `probe` async. The blocking-I/O wrapper changes the calculus.
