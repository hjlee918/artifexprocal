# LG OLED LUT Upload Binary Format — Research Notes

**Date:** 2026-04-30
**Status:** Incomplete — requires reverse-engineering against real TV or reference implementation
**References:** bscpylgtv, ColorControl, openlgtv gist, LG service menu behavior

---

## What We Know

### Upload Endpoint

LG OLEDs accept calibration data via:

```json
{
  "type": "request",
  "uri": "ssap://externalpq/setExternalPqData",
  "payload": {
    "data": "<base64-encoded-binary>",
    "picMode": "expert1",
    "colorSpace": "bt709"   // optional for 3D LUT
  }
}
```

The `data` field is a **base64-encoded binary blob**, not JSON. The blob's internal format is undocumented by LG.

### Chip Differences

| Chip | 1D LUT | 3D LUT |
|------|--------|--------|
| Alpha 9 Gen 1–3 (B9/C9) | 1024 entries | 17×17×17 |
| Alpha 9 Gen 4+ (C1/C2/C3/G2/G3) | 1024 entries | 33×33×33 |
| Alpha 7 (A1/B1) | 1024 entries | 17×17×17 |

1D LUT is always 1024 entries × 3 channels (R, G, B).

### Calibration Mode Requirements

- `start_calibration(picMode)` must be called before upload
- Calibration mode bypasses HDR10 tone mapping and disables ASBL
- LUTs are picture-mode specific (SDR, HDR10, Dolby Vision are independent)
- Uploading HDR10 LUT requires HDR10 signal on input
- Uploading DV config requires DV content playing

---

## What We Don't Know (Critical Gaps)

### 1. SDC Format vs Raw Binary

The `setExternalPqData` payload may be:
- **Raw binary:** Concatenated 15-bit or 16-bit channel values
- **SDC (Spectral Data Container):** A structured header + data format used by LG's internal tools
- **Dolby Vision config format:** XML or proprietary binary for DV uploads

**No official documentation exists.** The community has reverse-engineered this through:
- Packet capture of CalMAN ↔ LG TV communication
- Disassembly of LG webOS calibration service (`com.webos.service.tv.display`)
- bscpylgtv Python implementation
- ColorControl C# source code

### 2. Bit Depth and Packing

| Question | Current Guess | Status |
|----------|--------------|--------|
| Bit depth | 15-bit (0–32767) or 16-bit (0–65535)? | **Unverified** |
| Byte order | Little-endian (Intel) or big-endian? | **Unverified** |
| Packing | 2 bytes per sample, or 15-bit packed (e.g., 3×5 bytes for RGB)? | **Unverified** |
| 1D LUT structure | 1024 R values, then 1024 G, then 1024 B? Or interleaved RGB? | **Unverified** |
| 3D LUT structure | RGB triples in R-major order? B-major? | **Unverified** |
| Gamma table | Is the 1D LUT a gamma curve (input→output) or an inverse gamma (output→input)? | **Unverified** |

### 3. v1 Implementation Was Wrong

```rust
// archive/v1/crates/hal-displays/src/calibration_commands.rs
pub fn encode_1d_lut(lut: &Lut1D) -> Vec<u8> {
    let mut data = Vec::with_capacity(lut.size * 3 * 8);
    for ch in 0..3 {
        for &val in &lut.channels[ch] {
            data.extend_from_slice(&val.to_le_bytes()); // 64-bit float!
        }
    }
    data
}
```

This encodes LUT values as **64-bit IEEE 754 floats** (`f64::to_le_bytes()`). A real LG TV will reject this. The actual format is likely **16-bit unsigned integers** (or 15-bit) representing normalized values (0–32767 or 0–65535).

---

## Reference Implementations to Study

### bscpylgtv (Python)

Repository: https://github.com/chros73/bscpylgtv

Key file: `bscpylgtv/webos_client.py` — search for `setExternalPqData` and `calibration_data`

What to look for:
- How is the `data` field constructed before base64 encoding?
- What is the `calibration_data` structure?
- Does it use numpy arrays with specific dtype (`uint16`, `uint32`)?
- Is there a header prefix before the LUT values?

### ColorControl (C#)

Repository: https://github.com/Maassoft/ColorControl

Key files:
- `ColorControl.Lg/Controls/LgDevice.cs`
- `ColorControl.Lg/Services/LgService.cs`
- Search for `Upload1DLut`, `Upload3DLut`, `SetExternalPqData`

What to look for:
- Binary format of `byte[]` passed to the upload method
- How 1D LUT vs 3D LUT formatting differs
- Any header bytes or checksums

### openlgtv Gist

URL: https://gist.github.com/Informatic/1983f2e501444cf1cbd182e50820d6c1

Search for:
- `externalpq`
- `setExternalPqData`
- Calibration blob format

---

## LG Service Menu Observed Behavior

From user testing with CalMAN and real LG OLEDs:

1. **1D LUT upload** (SDR):
   - 1024 entries per channel
   - Upload changes grayscale tracking immediately
   - Upload does NOT change white balance (that uses separate `setWhiteBalance` command)

2. **3D LUT upload** (BT.709):
   - 33×33×33 × 3 channels on Alpha 9 Gen 4+
   - Upload changes color gamut tracking
   - TV requires reboot or mode switch to fully apply

3. **Dolby Vision config**:
   - Uses a different endpoint: `ssap://externalpq/setDolbyVisionConfigData`
   - Binary format is completely different from 1D/3D LUT
   - Contains white/black level, RGB trim, and tone mapping metadata

---

## Recommended Next Steps

### Immediate

1. **Study bscpylgtv source code** — Find the exact byte construction for `setExternalPqData`
2. **Study ColorControl source code** — Find C# equivalent
3. **Capture real traffic** — Use Wireshark or mitmproxy to observe CalMAN ↔ LG TV `setExternalPqData` payloads

### Before Rebuilding HAL-Displays

4. **Write a test script** (Python or Rust) that sends a known-good LUT to a real TV and verifies acceptance
5. **Document the confirmed binary format** in this file
6. **Write a `probe()` or `self_test()` method** for `LgOledController` that uploads a neutral LUT and reads it back (if possible)

### Open Questions

- [ ] What is the exact byte layout of the 1D LUT payload?
- [ ] What is the exact byte layout of the 3D LUT payload?
- [ ] What is the `data` field format for Dolby Vision config?
- [ ] Does LG validate checksums or magic headers?
- [ ] What error response does the TV return for malformed data?
- [ ] Can we read back uploaded LUTs for verification?

---

## Notes for v2 Architecture

The `hal-displays` crate's `DisplayController` trait needs a method like:

```rust
pub trait DisplayController {
    fn upload_1d_lut(&mut self, pic_mode: &str, lut: &Lut1D) -> Result<(), DisplayError>;
    fn upload_3d_lut(&mut self, pic_mode: &str, color_space: &str, lut: &Lut3D) -> Result<(), DisplayError>;
    fn upload_dolby_vision_config(&mut self, config: &DvConfig) -> Result<(), DisplayError>;
}
```

But the **encoding** of `Lut1D` and `Lut3D` into the LG binary format must be:
- Extracted from real reference code
- Unit-tested against captured payloads
- Versioned per chip/firmware (Alpha 9 Gen 4 may differ from Gen 3)

**Do not implement LG LUT upload in v2 until this document is marked COMPLETE.**
