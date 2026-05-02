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
