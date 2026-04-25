use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsapMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub uri: String,
    pub payload: serde_json::Value,
}

impl SsapMessage {
    pub fn start_calibration(pic_mode: &str) -> Self {
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://externalpq/startCalibration".to_string(),
            payload: serde_json::json!({"picMode": pic_mode}),
        }
    }

    pub fn end_calibration(pic_mode: &str) -> Self {
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://externalpq/endCalibration".to_string(),
            payload: serde_json::json!({"picMode": pic_mode}),
        }
    }

    pub fn upload_1d_lut(pic_mode: &str, data: &[u8]) -> Self {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://externalpq/setExternalPqData".to_string(),
            payload: serde_json::json!({
                "picMode": pic_mode,
                "data": STANDARD.encode(data),
            }),
        }
    }

    pub fn upload_3d_lut(pic_mode: &str, color_space: &str, data: &[u8]) -> Self {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://externalpq/setExternalPqData".to_string(),
            payload: serde_json::json!({
                "picMode": pic_mode,
                "colorSpace": color_space,
                "data": STANDARD.encode(data),
            }),
        }
    }

    pub fn set_white_balance(r_gain: u16, g_gain: u16, b_gain: u16) -> Self {
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://externalpq/setWhiteBalance".to_string(),
            payload: serde_json::json!({
                "rGain": r_gain,
                "gGain": g_gain,
                "bGain": b_gain,
            }),
        }
    }

    pub fn set_picture_mode(mode: &str) -> Self {
        Self {
            msg_type: "request".to_string(),
            uri: "ssap://system.launcher/launch".to_string(),
            payload: serde_json::json!({"id": mode}),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SsapResponse {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub id: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl SsapResponse {
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.msg_type == "response"
    }
}
