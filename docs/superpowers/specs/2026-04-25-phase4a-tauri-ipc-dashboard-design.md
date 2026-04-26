# Phase 4a: Tauri IPC + Dashboard Shell Design

> **Scope:** Backend IPC layer (commands + events) and frontend dashboard shell layout. No calibration wizard logic yet — just the infrastructure and navigation shell.

## Section 1: Architecture & IPC Layer

### Backend: CalibrationService Singleton

A single `CalibrationService` struct lives in the Tauri app state. It owns:
- Current meter (Box<dyn Meter>)
- Current display (Box<dyn DisplayController>)
- Current pattern generator (Box<dyn PatternGenerator>)
- Session history (in-memory + SQLite persistence)
- Calibration state machine (Idle → Connecting → Measuring → ...)

Tauri commands borrow `State<CalibrationService>` and call methods on it. The service emits Tauri events for state changes.

### IPC Pattern

**Commands** (request/response) for stateless operations:
- `get_app_state()` → full state snapshot
- `connect_meter(id)` / `disconnect_meter(id)`
- `connect_display(id)` / `disconnect_display(id)`
- `get_device_inventory()` → available devices
- `get_session_history()` → past calibrations

**Events** (server-sent) for streaming updates:
- `device-status-changed` — meter/display connect/disconnect
- `calibration-state-changed` — state machine transitions
- `error-occurred` — async errors surfaced to UI

### Type Safety

All shared types derive `specta::Type` for automatic TypeScript generation via `tauri-specta`. Frontend imports from auto-generated `src/bindings.ts` — no manual string typing of command names.

## Section 2: Dashboard Shell Component Structure

The dashboard shell is the root layout that persists across all views. It consists of four regions:

**1. Collapsible Left Sidebar**
- Width: 64px collapsed, 240px expanded
- Sections: Calibration, Devices, History, Settings
- Each item shows an icon + label (label hidden when collapsed)
- Active route highlighted with a subtle accent border
- Collapse/expand toggle at the bottom, keyboard shortcut `Cmd/Ctrl+B`

**2. Top Bar**
- Height: 48px
- Left: Current view title + breadcrumb
- Center: Live status indicators (meter connected/disconnected, display connected, current session state)
- Right: Global actions (emergency stop, notifications bell)

**3. Main Content Area**
- Fills remaining viewport
- Routes: `/dashboard` (overview), `/calibrate` (wizard), `/devices`, `/history`, `/settings`
- All views rendered within this area

**4. Status Footer**
- Height: 32px
- Shows: Last measurement value (if any), calibration progress bar during active session, version + build info

**Component tree:**
```
App
└── DashboardShell
    ├── Sidebar
    ├── TopBar
    ├── MainContent (react-router Outlet)
    └── StatusFooter
```

**State flow:** `DashboardShell` subscribes to Tauri events (`device-status-changed`, `calibration-state-changed`) and passes derived state down to `TopBar` and `StatusFooter` as props. `MainContent` handles its own data fetching via Tauri commands.

## Section 3: Tauri Command & Type Definitions

### Shared types (Rust ↔ TypeScript via Tauri Specta)

```rust
#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct MeterInfo {
    pub id: String,           // "i1-display-pro" | "i1-pro-2"
    pub name: String,
    pub serial: Option<String>,
    pub connected: bool,
    pub capabilities: Vec<String>, // ["emissive", "xyz", "spectrum"]
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub model: String,
    pub connected: bool,
    pub picture_mode: Option<String>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug)]
pub struct AppState {
    pub meters: Vec<MeterInfo>,
    pub displays: Vec<DisplayInfo>,
    pub calibration_state: CalibrationState,
    pub last_error: Option<String>,
}

#[derive(Serialize, Deserialize, specta::Type, Clone, Debug, Default)]
pub enum CalibrationState {
    #[default]
    Idle,
    Connecting,
    Measuring,
    GeneratingLut,
    Uploading,
    Verifying,
    Error,
}
```

### Tauri commands

- `get_app_state()` → `Result<AppState, String>`
- `connect_meter(meter_id: String)` → `Result<MeterInfo, String>`
- `disconnect_meter(meter_id: String)` → `Result<(), String>`
- `connect_display(display_id: String)` → `Result<DisplayInfo, String>`
- `disconnect_display(display_id: String)` → `Result<(), String>`
- `get_device_inventory()` → `Result<Vec<DeviceInfo>, String>`

### Events emitted from backend

- `device-status-changed` — `{ device_id, connected, info }`
- `calibration-state-changed` — `{ old_state, new_state, message }`
- `error-occurred` — `{ severity, message, source }`

## Section 4: Error Handling & Testing Strategy

### Backend error handling

`CalibrationError` enum with `thiserror::Error` derives user-friendly Display messages. All commands return `Result<T, String>` where the String is the Display representation. Frontend shows these directly in toast notifications.

### Frontend error handling

- Global error boundary wraps `MainContent`
- `useTauriError` hook subscribes to `error-occurred` events
- Inline error states on triggering components
- Emergency stop button calls `abort_calibration()`

### Testing strategy

| Layer | Approach |
|-------|----------|
| Rust commands | Unit tests with mocked CalibrationService |
| Rust events | Integration tests in Tauri test mode |
| TypeScript bindings | CI check that `bindings.ts` is up-to-date |
| Frontend components | Vitest + React Testing Library |
| E2E | Playwright against Tauri WebDriver |

Hardware-in-the-loop tests remain `#[ignore]` and run manually.
