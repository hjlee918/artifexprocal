# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **greenfield project** for a professional-grade display calibration software suite competing with CalMAN Ultimate and Light Illusion's ColourSpace INF. The design spec is at `docs/superpowers/specs/2026-04-24-calibration-software-design.md`.

**Planned tech stack:** Tauri (Rust backend + React/TypeScript frontend). The project has not been scaffolded yet — there is no build system, no `package.json`, no `Cargo.toml`.

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

### Supported File Formats (planned)
- LUTs: `.cube`, `.3dl`, `.xml` (Dolby Vision), `.dat` (madVR)
- Profiles: `.icc`/`.icm`, `.ccmx`, `.ccss`
- Data: `.csv`, `.json`

## Architecture (planned)

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
- `ipc` — Tauri command/event handlers

### Frontend (React + TypeScript)
- **Visualization:** Three.js/WebGL for CIE diagrams and 3D LUT cubes
- **Wizards:** Step-by-step calibration flows (AutoCal, Manual, 3D LUT, Profiling)
- **Dashboard:** Session history, device inventory, quick actions
- **State:** Zustand for global state, React Query for backend sync

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

## Repository

- **GitHub:** https://github.com/hjlee918/artifexprocal.git
- **Remote name:** `origin`
- **Push on every change:** The user wants all progress committed and pushed immediately

## When Scaffolding

When this project gets initialized:
1. Use `npm create tauri-app@latest` or `cargo create-tauri-app` to scaffold
2. The Rust workspace should use a single `Cargo.toml` with multiple crates (not a monorepo of separate packages)
3. Frontend should be React 19 + TypeScript + Tailwind CSS + Vite
4. Add Three.js for visualization
5. SQLite via `rusqlite` or `sqlx` for the Rust backend
6. Prefer Rust crates for color science over calling into Python (`colour-science` is Python-only)

## LG OLED Development Resources

These are essential references for understanding the LG webOS internals, network control protocols, and how to communicate with the panel directly:

- [openlgtv webOS hacking notes](https://gist.github.com/Informatic/1983f2e501444cf1cbd182e50820d6c1) — Reverse-engineered webOS internals, calibration API endpoints, service commands
- [bscpylgtv](https://github.com/chros73/bscpylgtv) — Python library for controlling webOS-based LG TVs over the network. Shows the exact websocket/HTTP commands for picture mode switching, calibration data upload, and system service calls
- [webOS Open Source Edition docs](https://www.webosose.org/docs/home/) — Official webOS OSE documentation for understanding the OS architecture, service APIs, and luna-service bus
- [ColorControl (Maassoft)](https://github.com/Maassoft/ColorControl) — C# application that controls LG OLEDs and NVIDIA GPUs. Excellent reference for the LG calibration protocol, including 1D/3D LUT upload formats, white balance commands, and HDR tone curve settings

These are the primary references for implementing the `hal_displays` LG OLED module.

## Competitors for Reference

- [CalMAN Ultimate](https://store.portrait.com/calman-ultimate/) — $2,995, Windows-only, subscription updates
- [CalMAN Home for LG](https://store.portrait.com/calman-home-for-lg.html) — ~$145, consumer-focused
- [ColourSpace INF](https://www.lightillusion.com/colourspace.html) — lifetime license, volumetric 3D graphs, no subscription

If asked to add features, compare against these tools' capabilities rather than reinventing from first principles.
