use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItpgMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub uri: String,
    pub payload: serde_json::Value,
}

impl ItpgMessage {
    pub fn enable(on: bool) -> Self {
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://com.webos.service.tv.display/displayPattern".to_string(),
            payload: serde_json::json!({
                "pattern": "color",
                "enabled": on,
            }),
        }
    }

    pub fn set_patch_color(r: u16, g: u16, b: u16) -> Self {
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://com.webos.service.tv.display/displayPattern".to_string(),
            payload: serde_json::json!({
                "pattern": "color",
                "r": r,
                "g": g,
                "b": b,
            }),
        }
    }

    pub fn set_window(win_h: u16, win_v: u16, patch_h: u16, patch_v: u16) -> Self {
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://com.webos.service.tv.display/displayPattern".to_string(),
            payload: serde_json::json!({
                "pattern": "color",
                "windowH": win_h,
                "windowV": win_v,
                "patchH": patch_h,
                "patchV": patch_v,
            }),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Convert 8-bit (0-255) RGB to 10-bit (0-1023) for iTPG
pub fn to_10bit(val: f64) -> u16 {
    (val.clamp(0.0, 1.0) * 1023.0).round() as u16
}
