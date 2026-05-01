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
