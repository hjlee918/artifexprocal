# Phase 3b: Network Protocols Design Spec

> **Date:** 2026-04-24  
> **Scope:** LG OLED AutoCal protocol (SSAP/WebSocket) and PGenerator 1.6 pattern generator (TCP XML / HTTP / DeviceControl)  
> **Approach:** reqwest::blocking, dual-mode for both devices

---

## 1. Goal

Implement real network protocol drivers for the two primary hardware devices in the calibration loop:
- **LG OLED TV** — SSAP over WebSocket for calibration commands, LUT upload, and iTPG control
- **PGenerator 1.6** — TCP XML protocol (LightSpace Network Calibration Protocol) for patch display and pattern sequencing

Both devices support **dual-mode operation**:
- **Direct mode** — Our app talks directly to the device (offline-first, no external dependencies)
- **DeviceControl mode** — Our app talks to LightSpace DeviceControl running locally, which proxies commands to the device

---

## 2. Architecture

### New Crates

| Crate | Responsibility | Devices |
|-------|---------------|---------|
| `hal-displays` | LG OLED display controller | LG OLED (direct + DeviceControl) |
| `hal-patterns` | Pattern generator controller | PGenerator 1.6 (direct + DeviceControl) |

### Crate Dependencies

```
hal-displays
├── hal (traits + types)
├── calibration-core (SessionConfig, errors)
├── reqwest (blocking HTTP for DeviceControl)
└── tungstenite (WebSocket for direct SSAP)

hal-patterns
├── hal (traits + types)
├── calibration-core
├── reqwest (blocking HTTP for DeviceControl)
└── quick-xml (XML serialization for direct TCP)
```

---

## 3. PGenerator Pattern Generator

### 3.1 Dual-Mode Architecture

```rust
pub enum PGeneratorMode {
    Direct { ip: String, port: u16 },      // TCP port 85 (XML)
    DeviceControl { port: u16 },           // HTTP localhost:81
}

pub struct PGeneratorController {
    mode: PGeneratorMode,
    connected: bool,
    stream: Option<TcpStream>,  // Direct mode only
}
```

### 3.2 Direct Mode — TCP Port 85 (LightSpace XML Protocol)

**Connection:** TCP socket to `pi-ip:85`

**XML Commands:**
```xml
<!-- Display solid color patch (real-time) -->
<patch>
  <r>128</r>
  <g>64</g>
  <b>32</b>
</patch>

<!-- Play pre-encoded disk pattern by name -->
<pattern>
  <name>21-Point Grayscale</name>
  <chapter>1</chapter>
</pattern>

<!-- Black patch (disconnect) -->
<patch>
  <r>0</r>
  <g>0</g>
  <b>0</b>
</patch>
```

**Response:** XML acknowledgment or error

### 3.3 DeviceControl Mode — HTTP localhost:81

**Endpoints (assumed, verify against DeviceControl docs):**
```
POST /api/display_pattern
  Body: {"name": "21-Point Grayscale"}

POST /api/display_patch
  Body: {"r": 128, "g": 64, "b": 32}

GET /api/list_patterns
  Response: ["Brightness Patterns", "21-Point Grayscale", "Color Checker Classic (24 Colors)", ...]
```

### 3.4 Ted's LightSpace Disk Pattern Catalog

Pre-Calibration Tools:
- Brightness Patterns
- Contrast Patterns
- Color Temperature pattern
- Black & White Pattern
- Sharpness Pattern

Display Characterization:
- 21-Point Grayscale
- 20-Steps per Primary Color (RGB)
- 20-Steps per Primary & Secondary Color (RGBCMY)
- 10-Point Cube (1,000 points)
- 10-Point Cube Hybrid 1D+3D (1,021 points)
- 17-Point Cube (4,913 points)
- 17-Point Cube Hybrid 1D+3D (4,934 points)
- 21-Point Cube (9,261 points)

Verification:
- Color Checker Classic (24 Colors)
- Color Checker SG (96 Colors)
- Color Checker SG Fleshtones (19 Colors)
- 6-Point Near Black Patterns (0.5-1-2-3-4-5%)
- 2/3/4-Point Grayscale
- 11/21-Point Grayscale Calibration
- 4/5/10-Point Saturation (25/31/61 Colors)
- 4/5/10-Point Luminance (28/35/70 Colors)
- Color Checker (24 Colors)
- Color Checker Skin Tones (19 Colors)
- Contrast Ratio Patterns

---

## 4. LG OLED Display Controller

### 4.1 Dual-Mode Architecture

```rust
pub enum LgOledMode {
    Direct { ip: String, port: u16 },       // WebSocket 3000/3001
    DeviceControl { port: u16 },            // HTTP localhost:81
}

pub struct LgOledController {
    mode: LgOledMode,
    connected: bool,
    paired: bool,
    client_key: Option<String>,
    calibration_active: bool,
    ws_stream: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}
```

### 4.2 Direct Mode — SSAP over WebSocket

**Discovery:** SSDP M-SEARCH to `239.255.255.250:1900`
```
M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: "ssdp:discover"
ST: urn:lge-com:service:webos-second-screen:1
MX: 2
```

**Pairing Flow:**
1. Connect WebSocket to `ws://tv-ip:3000`
2. Register app with `ssap://com.webos.service.tvpairing/getKey`
3. TV displays PIN on screen
4. Send PIN via WebSocket
5. TV responds with `clientKey` — store for future connections

**Calibration Protocol:**
```json
// Enter calibration mode
{"type": "request", "uri": "ssap://externalpq/startCalibration", 
 "payload": {"picMode": "expert1"}}

// Upload 1D LUT
{"type": "request", "uri": "ssap://externalpq/setExternalPqData",
 "payload": {"data": "<base64-encoded-lut>", "picMode": "expert1"}}

// Upload 3D LUT (BT.709)
{"type": "request", "uri": "ssap://externalpq/setExternalPqData",
 "payload": {"data": "<base64-encoded-3d-lut>", "picMode": "expert1", "colorSpace": "bt709"}}

// Set white balance
{"type": "request", "uri": "ssap://externalpq/setWhiteBalance",
 "payload": {"rGain": 128, "gGain": 128, "bGain": 128}}

// Exit calibration mode
{"type": "request", "uri": "ssap://externalpq/endCalibration",
 "payload": {"picMode": "expert1"}}
```

**iTPG Control (2019+ models):**
```json
// Enable iTPG
{"type": "request", "uri": "ssap://com.webos.service.tv.display/displayPattern",
 "payload": {"pattern": "color", "enabled": true}}

// Set patch color (10-bit)
{"type": "request", "uri": "ssap://com.webos.service.tv.display/displayPattern",
 "payload": {"pattern": "color", "r": 512, "g": 512, "b": 512}}
```

### 4.3 DeviceControl Mode — HTTP localhost:81

DeviceControl handles all WebSocket/SSAP complexity. Our app sends high-level commands:

```
POST /api/lg/connect
  Body: {"ip": "192.168.1.100", "port": 3000}

POST /api/lg/start_calibration
  Body: {"picMode": "expert1"}

POST /api/lg/upload_1d_lut
  Body: {"data": "<base64>", "picMode": "expert1"}

POST /api/lg/upload_3d_lut
  Body: {"data": "<base64>", "picMode": "expert1", "colorSpace": "bt709"}

POST /api/lg/set_white_balance
  Body: {"rGain": 128, "gGain": 128, "bGain": 128}

POST /api/lg/end_calibration
  Body: {"picMode": "expert1"}
```

### 4.4 Calibration Mode Requirements

Per the protocol constraints:
- Calibration commands only work during calibration mode (between `start_calibration` and `end_calibration`)
- HDR10 tone mapping is bypassed during calibration mode
- Calibration data is picture-mode specific (SDR, HDR10, Dolby Vision are independent)
- To upload HDR10 LUTs, TV must be receiving HDR10 signal
- To upload Dolby Vision configs, TV must be playing DV content
- Alpha 9 Gen 4+ chips use 33-point 3D LUTs; Alpha 7 uses 17-point

---

## 5. HAL Trait Implementations

Both controllers implement existing HAL traits so they drop into `GreyscaleAutoCalFlow` without changes.

### PGeneratorController

```rust
impl PatternGenerator for PGeneratorController {
    fn connect(&mut self) -> Result<(), PatternGenError>;
    fn disconnect(&mut self);
    fn display_patch(&mut self, color: &RGB) -> Result<(), PatternGenError>;
}
```

**Extension trait for pattern names:**
```rust
pub trait PatternGeneratorExt: PatternGenerator {
    fn display_pattern(&mut self, pattern_name: &str) -> Result<(), PatternGenError>;
    fn list_patterns(&self) -> Vec<String>;
}
```

### LgOledController

```rust
impl DisplayController for LgOledController {
    fn connect(&mut self) -> Result<(), DisplayError>;
    fn disconnect(&mut self);
    fn set_picture_mode(&mut self, mode: &str) -> Result<(), DisplayError>;
    fn upload_1d_lut(&mut self, lut: &Lut1D) -> Result<(), DisplayError>;
    fn upload_3d_lut(&mut self, lut: &Lut3D) -> Result<(), DisplayError>;
    fn set_white_balance(&mut self, gains: RGBGain) -> Result<(), DisplayError>;
}
```

---

## 6. Error Handling

### PGeneratorError

```rust
pub enum PGeneratorError {
    ConnectionFailed(String),
    XmlParseError(String),
    HttpError(String),
    PatternNotFound { name: String },
    DeviceControlUnavailable,
    InvalidResponse(String),
}
```

### LgOledError

```rust
pub enum LgOledError {
    DiscoveryFailed(String),
    PairingRejected(String),
    WebSocketError(String),
    CalibrationModeError(String),
    UploadFailed { reason: String },
    DeviceControlUnavailable,
    InvalidResponse(String),
}
```

Both map to `PatternGenError` and `DisplayError` respectively for HAL trait compatibility.

---

## 7. Testing Strategy

### Unit Tests

- **Mock TCP server** (port 85): Responds to XML commands with acknowledgments
- **Mock WebSocket server** (port 3000): Responds to SSAP JSON
- **Mock HTTP server** (port 81): Simulates DeviceControl API

### Integration Tests

- `test_pgenerator_direct_patch_display`: Connect to mock TCP server, send RGB patch, verify response
- `test_pgenerator_devicecontrol_pattern`: Connect to mock HTTP server, request pattern by name
- `test_lg_discovery`: Send SSDP M-SEARCH, verify mock TV responds
- `test_lg_pairing_flow`: Full PIN pairing sequence with mock TV
- `test_lg_calibration_upload`: Enter calibration mode, upload 1D LUT, verify acknowledgment
- `test_dual_mode_switching`: Create controller in direct mode, switch to DeviceControl, verify behavior

### Hardware Tests (manual, not automated)

- `test_real_pgenerator_direct`: Connect to actual Raspberry Pi on local network
- `test_real_lg_pairing`: Pair with LG OLED TV, verify PIN flow
- `test_real_lg_calibration`: Full calibration mode entry/exit on real TV

---

## 8. UI Integration (Future Phase)

Settings panel will expose mode selector per device:

```
PGenerator:
  [ ] Direct (TCP)        IP: [192.168.1.200]  Port: [85]
  [ ] DeviceControl       Port: [81]

LG OLED:
  [ ] Direct (WebSocket)  IP: [192.168.1.100]  Port: [3000]
  [ ] DeviceControl       Port: [81]
```

Both modes equally accessible. Default: Direct mode.

---

## 9. File Structure

```
crates/hal-patterns/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── pgenerator.rs          # PGeneratorController
    ├── xml_protocol.rs        # XML serialization for direct mode
    ├── devicecontrol_client.rs # HTTP client for DeviceControl mode
    └── patterns_catalog.rs    # Ted's disk pattern names

crates/hal-displays/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── lg_oled.rs             # LgOledController
    ├── ssap_protocol.rs       # SSAP JSON/WebSocket for direct mode
    ├── discovery.rs           # SSDP discovery
    ├── pairing.rs             # PIN-based pairing flow
    ├── calibration_commands.rs # start/end/upload LUT
    ├── itpg.rs                # Internal pattern generator control
    └── devicecontrol_client.rs # HTTP client for DeviceControl mode
```

---

## 10. Spec Self-Review

- **Placeholder scan:** No TBD/TODO placeholders. All endpoints are documented with assumed shapes (may need adjustment against real DeviceControl API).
- **Internal consistency:** Both devices follow the same dual-mode pattern. HAL traits are unchanged.
- **Scope check:** Focused on protocol implementation only. UI integration is noted as future work.
- **Ambiguity check:** DeviceControl API endpoints are assumed based on common REST patterns. Verification against real DeviceControl required.

---

## Appendix: PGenerator+ HTTP API (Reference Only)

If the user has PGenerator+ (community fork) instead of PGenerator 1.6, these additional endpoints are available:

```
GET  /api/ping          Health check
GET  /api/info          Device status
GET  /api/config        Current configuration
POST /api/config        Apply configuration
GET  /api/modes         Available HDMI modes
POST /api/pattern       Display pattern (JSON body)
GET  /api/infoframes    Read AVI/DRM InfoFrame data
```

PGenerator+ runs a web UI on port 80 and supports HDR10/HLG/Dolby Vision.

The direct TCP port 85 XML protocol remains the primary integration path for maximum compatibility.
