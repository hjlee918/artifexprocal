# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working on the ArtifexProCal project.

## Project Overview

ArtifexProCal is a **professional-grade display calibration software suite** competing with CalMAN Ultimate and Light Illusion's ColourSpace INF. The design spec is at `docs/superpowers/specs/2026-04-24-calibration-software-design.md`.

**Tech stack:** Tauri 2.x (Rust backend + React/TypeScript frontend). The project is currently on the `clean-slate` branch — all v1 code is archived at `archive/v1/`.

**Current status:** v1 has been archived. v2 rebuild is in early design phase. The build infrastructure (`package.json`, `Cargo.toml`, `vite.config.ts`, `tauri.conf.json`) is intact; application code is minimal placeholder.

---

## Lessons from v1 (Critical — Do Not Repeat)

See `docs/LESSONS_LEARNED.md` for the full retrospective. Key rules:

1. **Store hydrates on mount** — Zustand (or any frontend store) must fetch backend state on component mount. Never assume events have already fired.
2. **IPC handlers are thin delegators** — Under 100 lines each. Business logic lives in focused engine crates, not `CalibrationService`.
3. **Bindings auto-generated** — `tauri-specta` generates TypeScript bindings on every `cargo build`. Hand-maintained `bindings.ts` is forbidden.
4. **State machine before UI** — Define the wizard/session state machine first. Every step receives the same state object and emits the same action enum.
5. **No Three.js until real data flows** — Start with SVG/Canvas for 2D diagrams. Three.js is Phase 5+.
6. **No `blocking_recv` in UI-facing code** — Use async events with timeouts and explicit termination conditions.
7. **Pattern generator is a first-class device** — Must be discoverable, connectable, and visible in the device list. No implicit auto-connect.
8. **Every phase ships end-to-end** — Each phase must produce a working, testable increment. No integration-only phases.
9. **Correction matrix is HAL concern** — Applied at the `Meter` trait level, not duplicated across calibration flows.
10. **Hardware drivers need self-test** — Every driver must have a `probe()` method for connectivity verification.

---

## Domain Context

### What This Software Does
Professional display calibration involves measuring a display's color accuracy with a colorimeter or spectrophotometer, then generating correction data (1D LUTs, 3D LUTs, or direct display settings) to make the output match a target color space.

Key concepts:
- **Color spaces:** BT.709 (SDR), BT.2020 (HDR), DCI-P3
- **Color difference metrics:** DeltaE 2000 (the industry standard perceptual metric)
- **Tone curves:** Gamma 2.2/2.4, PQ (ST.2084), HLG
- **LUTs:** 1D LUTs (per-channel tone curves, typically 256–1024 entries) and 3D LUTs (volumetric color correction cubes, typically 17³ to 65³)
- **CIE diagrams:** xy (1931) and u'v' (1976) chromaticity diagrams for visualizing gamut triangles
- **Meter profiling:** Creating a correction matrix for a colorimeter against a spectrophotometer reference, since colorimeters have filter mismatch on certain spectra (especially OLED)

### User's Hardware Lab
The user owns and will use for development/testing:
- **Displays:** LG OLED TV (2018–2025/26 models with network AutoCal), Sony VPL-VW385ES projector
- **Meters:** X-Rite i1 Display Pro Rev.B (2000 nits HDR capable), X-Rite i1 Pro 2 spectrophotometer
- **Pattern generators:** PGenerator 1.6 on Raspberry Pi 4, Ted's LightSpace CMS Calibration Disk templates, LG internal pattern generator (2019+)
- **Existing software for reference:** CalMAN Business 5.12, ColourSpace ZRO, ArgyllPro ColorMeter 2 on Android tablet

### LG AutoCal Protocol
LG OLEDs from 2018+ support calibration over the local network via an HTTP API. The TV displays a passcode for pairing. The software uploads 1D LUTs, 3D LUTs, and white balance settings directly. This is the primary calibration path for the MVP.

**Known limitation:** The exact binary format for LG LUT upload (`setExternalPqData`) is **not fully reverse-engineered**. See `docs/LG_LUT_FORMAT.md` for the research status. Do not implement LG LUT upload until this is resolved.

### Supported File Formats (planned)
- LUTs: `.cube`, `.3dl`, `.xml` (Dolby Vision), `.dat` (madVR)
- Profiles: `.icc`/`.icm`, `.ccmx`, `.ccss`
- Data: `.csv`, `.json`

---

## Architecture (planned — v2)

### Backend (Rust)
Modular crate structure:
- `color_science` — XYZ/Lab/LCh/ICtCp conversions, DeltaE, gamut math, tone curves
- `hal` — Hardware abstraction traits (`Meter`, `DisplayController`, `PatternGenerator`)
- `hal_meters` — X-Rite i1 Display Pro (HID), i1 Pro 2
- `hal_displays` — LG OLED (HTTP AutoCal), Sony projector (RS-232/IP)
- `hal_patterns` — PGenerator (HTTP), LG internal
- `calibration` — Session manager, patch sequencer, measurement loop, AutoCal logic, 1D/3D LUT generation
- `profiling` — Display characterization, meter correction matrix generation
- `reporting` — PDF/HTML report generation
- `ipc` — Tauri command/event handlers (thin layer)

### Frontend (React + TypeScript)
- **Visualization:** SVG/Canvas for CIE diagrams and gamma curves; Three.js/WebGL deferred until Phase 5+
- **Wizards:** Step-by-step calibration flows (AutoCal, Manual, 3D LUT, Profiling)
- **Dashboard:** Session history, device inventory, quick actions
- **State:** Zustand for UI state; backend state is fetched explicitly, not assumed from events

---

## User Preferences

- **No subscriptions** — lifetime license model (like ColourSpace INF)
- **Cross-platform from day one** — macOS, Windows, Linux
- **Focused MVP:** LG OLED + Sony projector initially, expand to other brands later
- **Hardware-first development:** Real measurements with actual devices, not just mocks
- **User has full permissions** — `.claude/settings.local.json` uses `defaultMode: "dontAsk"`
- **License model:** Tiered (Lite / Pro / Ultimate), lifetime purchase, no subscription
- **Open source:** Color science backend and HAL traits are open-source; UI and hardware driver implementations may be proprietary
- **Offline-first:** Entirely offline, no account validation
- **Measurement mode:** Fast default with optional high-precision iterative mode

---

## Meter Driver Architecture (Critical for HAL)

### i1 Display Pro Rev.B

**macOS:** Uses ArgyllCMS `spotread -c 1` via PTY-based subprocess adapter (`argyll_adapter.rs`). Native HID works for unlock but subsequent commands hit IOKit write timeout — ArgyllCMS fallback is required.

**Linux/Windows:** Native HID via `hidapi` + challenge-response unlock protocol (`i1d3_unlock.rs`). 11 known OEM keys exist. `I1D3_ESCAPE` env var for unknown variants. Unlock protocol proven correct on physical hardware.

**ArgyllCMS dependency:** Cannot bundle due to AGPL. Must prompt user to install (`brew install argyll-cms`). Only required on macOS.

### i1 Pro 2

**macOS:** Uses ArgyllCMS `spotread -c 2`. Cannot do native USB due to `IOUSBHost` driver conflict.

**Linux/Windows:** Native USB bulk transfer via `rusb`.

### Routing Rule

| Platform | i1 Display Pro | i1 Pro 2 |
|----------|---------------|----------|
| macOS | ArgyllCMS | ArgyllCMS |
| Linux | Native HID | Native USB |
| Windows | Native HID | Native USB |

---

## Repository

- **GitHub:** https://github.com/hjlee918/artifexprocal.git
- **Remote name:** `origin`
- **Push on every change:** The user wants all progress committed and pushed immediately
- **Active branch:** `clean-slate` (do not merge to `main` until v2 is functional)

---

## Document Generation

**Always produce both `.md` and `.docx` formats.**

- `.md` stays in version control for diffs and code review
- `.docx` is for the user's consumption and reporting
- `.md` is the single source of truth; `.docx` should be generated from `.md` via a script, not hand-edited

---

## LG OLED Calibration Protocol (SSAP over WebSocket)

**Transport:** WebSocket connection on port 3000 (plain) or 3001 (secure/wss). Commands sent as cleartext JSON payloads.

**Discovery:** SSDP M-SEARCH to `udp://239.255.255.250:1900` with service type `urn:lge-com:service:webos-second-screen:1`. TV responds with its IP and WebSocket endpoint.

**Pairing:** PIN-based. TV displays a passcode on screen; client sends it via WebSocket to authenticate. Client key is stored for subsequent connections.

**Calibration Mode:** Must be entered before any calibration commands. Uses `start_calibration(picMode="expert1")` and `end_calibration()`. During calibration mode, HDR10 tone mapping is bypassed, ASBL is disabled, and the TV accepts LUT uploads.

**Key Calibration Commands:**
- `start_calibration(picMode)` — Enter calibration mode for a specific picture profile
- `end_calibration()` — Exit calibration mode and lock state
- `upload_1d_lut(data)` / `upload_1d_lut_from_file(path)` — Upload 1D LUT (1024 entries for SDR)
- `upload_3d_lut_bt709_from_file(path)` — Upload 3D LUT for BT.709 (33x33x33 on Alpha 9 Gen 4+)
- `upload_3d_lut_bt2020_from_file(path)` — Upload 3D LUT for BT.2020
- `set_dolby_vision_config_data(data)` — Upload Dolby Vision configuration
- `set_3by3_gamut_data(matrix)` — Upload 3x3 gamut correction matrix
- `set_tonemap_params(params)` — Set HDR10 tone mapping parameters
- `ddc_reset` — Reset DDC controls to factory defaults

**Picture Mode Independence:** SDR, HDR10, and Dolby Vision are completely independent. To upload a LUT for a specific mode, the TV must be receiving that signal type (e.g., HDR10 content playing for HDR10 LUT upload, DV blank video for DV config upload).

**Chip Differences:** Alpha 9 Gen 4 (C1) uses 33-point 3D LUTs; Alpha 7 uses 17-point. Model string available from `get_software_info` SSAP response.

### iTPG (Internal Test Pattern Generator)

Available on 2019+ LG OLED models. Accessible via SSAP/WebSocket during calibration mode.

**Functions:**
- `start_itpg()` — Enable internal pattern generator
- `stop_itpg()` — Disable internal pattern generator
- `set_itpg_patch_window(win_h, win_v, patch_h, patch_v)` — Set window and patch size
- `set_itpg_patch_color(r, g, b, ...)` — Set current patch color (10-bit values, 0-1023)

**Note:** iTPG operates at the TV's native bit depth. RGB values are 10-bit (0-1023). The iTPG cannot generate Dolby Vision metadata, so DV calibration cannot be verified using iTPG alone.

### PGenerator 1.6 HTTP API (External Pattern Generator)

PGenerator by LightSpace runs on Raspberry Pi 4 and accepts HTTP commands to display test patches on its HDMI output.

**Base URL:** `http://<pi-ip>:8080`

**Endpoints:**
- `GET /patch?r=<R>&g=<G>&b=<B>` — Display patch with 8-bit RGB values (0-255)
- `GET /patch?r=0&g=0&b=0` — Display black patch

**Implementation Notes:**
- PGenerator is always running; no explicit start/stop needed
- On stop, send black patch to clear the display
- Include a `probe()` method that calls the black endpoint to verify connectivity before measurement sessions
- If the actual API differs (e.g., `/measure` or `/setPatch`), only the PGenerator client implementation needs updating

---

## Pre-Calibration Procedures

**Equipment Warm-up:**
- TV: Powered on with standard content for minimum 45 minutes (preferably 1 hour)
- Probes: Connected to USB port of calibration computer for 20-30 minutes minimum

**TV Settings Preparation:**
- Disable processing (2021+ LG OLED models have specific processing disable steps)
- Disable ASBL (Auto Static Brightness Limiter) and GSR
- Set Brightness and Contrast to appropriate reference values
- Pre-calibrate white balance in Service Menu if desired
- For HDR: Play HDR blank video file via internal media player to maintain HDR mode
- For Dolby Vision: Play DV blank video file to maintain DV mode

**Measurement Setup:**
- Set stabilization delay to 5 seconds
- Set patch size to L32 (32% of screen)
- Enable Profile Luma (Nits) Auto
- Set patch scale to Legal for SDR, Full for HDR/DV
- Minimum extra delay time: 0.50 seconds to minimize sync read issues with iTPG

**Target Values (SDR Reference):**
- Color space: Rec.709
- Gamma: Power Law 2.4
- Peak luminance: 100 nits (@ 100% White)
- 109% white: ~124 nits (for Video Extended range with Contrast at default 85)

---

## Calibration Workflow

1. **Connect TV** — SSAP WebSocket connection with PIN pairing
2. **Select picture mode + color space + HDR format**
3. **Select pattern generator** — iTPG (internal) or PGenerator (external Pi)
4. **Select meter** — i1 Display Pro or i1 Pro 2
5. **Run pre-calibration measurement** — Display grayscale + primaries + secondaries, record XYZ readings
6. **Generate correction LUTs** — 1D tone curve from grayscale, 3D LUT from full patch set
7. **Upload LUTs** — Upload to TV via calibration API
8. **Verify** — Measure again to confirm calibration accuracy

---

## Key Open-Source Libraries for Reference

| Language | Library | URL |
|----------|---------|-----|
| Python (async) | bscpylgtv | https://github.com/chros73/bscpylgtv |
| Python (async) | aiopylgtv | https://github.com/bendavid/aiopylgtv |
| Python | PyWebOSTV | https://github.com/supersaiyanmode/PyWebOSTV |
| Node.js | lgtv2 | https://github.com/hobbyquaker/lgtv2 |
| Go | go-webos | https://pkg.go.dev/github.com/kaperys/go-webos |

---

## Firmware Warnings

- **webOS 7.3+:** Communication protocol changed and broke existing calibration tools. Do not update to webOS 7.3 if calibration compatibility is required.
- **Model/Year Differences:** The LG command protocol is inconsistent between models and firmware versions. Commands must be validated per model/year combination.

---

## Competitors for Reference

- [CalMAN Ultimate](https://store.portrait.com/calman-ultimate/) — $2,995, Windows-only, subscription updates
- [CalMAN Home for LG](https://store.portrait.com/calman-home-for-lg.html) — ~$145, consumer-focused
- [ColourSpace INF](https://www.lightillusion.com/colourspace.html) — lifetime license, volumetric 3D graphs, no subscription

If asked to add features, compare against these tools' capabilities rather than reinventing from first principles.
