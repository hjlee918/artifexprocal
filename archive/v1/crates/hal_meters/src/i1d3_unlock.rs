use crate::hid_util::{SyncHidDevice, HidUtilError, send_command_u16, read_response};

pub const CMD_LOCK_CHALLENGE: u16 = 0x9900;
pub const CMD_LOCK_RESPONSE: u16 = 0x9A00;
pub const CMD_RELOCK: u16 = 0x9B00;
pub const CMD_GET_LOCKED_STATUS: u16 = 0x0020;

/// Known 64-bit unlock keys for i1d3 OEM variants.
/// Each entry is (key[0], key[1]) as two u32 values.
const KNOWN_KEYS: &[(u32, u32)] = &[
    (0xe9622e9f, 0x8d63e133), // Retail i1 Display Pro
    (0xe01e6e0a, 0x257462de), // ColorMunki Display
    (0xcaa62b2c, 0x30815b61), // Generic OEM
    (0xa9119479, 0x5b168761), // NEC SpectraSensor Pro
    (0x160eb6ae, 0x14440e70), // Quato Silver Haze 3
    (0x291e41d7, 0x51937bdd), // HP DreamColor
    (0x1abfae03, 0xf25ac8e8), // Wacom
    (0x828c43e9, 0xcbb8a8ed), // TPA-1
    (0xe8d1a980, 0xd146f7ad), // Barco
    (0x171ae295, 0x2e5c7664), // Crysta (IODATA)
    (0x64d8c546, 0x4b24b4a7), // ViewSonic X-Rite i1
];

#[derive(Debug, thiserror::Error)]
pub enum UnlockError {
    #[error("HID communication failed: {0}")]
    Hid(String),
    #[error("Challenge request failed: status {status:02X}")]
    ChallengeFailed { status: u8 },
    #[error("Unlock response rejected: no key matched")]
    NoKeyMatched,
    #[error("Unlock verification failed: device still locked")]
    StillLocked,
}

impl From<HidUtilError> for UnlockError {
    fn from(e: HidUtilError) -> Self {
        UnlockError::Hid(e.to_string())
    }
}

/// Attempt to unlock the i1 Display Pro using the challenge-response protocol.
/// Tries all known keys, plus any user-supplied key from the I1D3_ESCAPE env var.
pub fn i1d3_unlock(device: &mut SyncHidDevice) -> Result<(), UnlockError> {
    // 1. Request challenge
    send_command_u16(device, CMD_LOCK_CHALLENGE, &[])?;
    let challenge = read_response(device, 2000)?;
    if challenge.len() < 43 {
        return Err(UnlockError::ChallengeFailed {
            status: challenge.first().copied().unwrap_or(0xFF),
        });
    }

    // Decode challenge bytes
    let xor_key = challenge[3];
    let mut sc = [0u8; 8];
    for i in 0..8 {
        sc[i] = challenge[35 + i] ^ xor_key;
    }

    // 2. Try built-in keys
    let mut keys: Vec<(u32, u32)> = KNOWN_KEYS.to_vec();

    // 3. Try user-supplied escape key
    if let Ok(escape_hex) = std::env::var("I1D3_ESCAPE") {
        if let Some(key) = parse_escape_key(&escape_hex) {
            keys.push(key);
        }
    }

    // 4. For each key, compute response and send it
    for &(k0, k1) in &keys {
        let response = create_unlock_response(sc, k0, k1);

        let mut report_payload = [0u8; 64];
        // Encode response at payload offset 22, so that send_command_u16
        // places it at HID report offset 24 (payload starts at report[2]).
        let encode_key = challenge[2];
        for i in 0..16 {
            report_payload[22 + i] = response[i] ^ encode_key;
        }

        send_command_u16(device, CMD_LOCK_RESPONSE, &report_payload)?;
        let verify = read_response(device, 2000)?;

        // Check response[2] == 0x77 for success
        if verify.len() > 2 && verify[2] == 0x77 {
            return Ok(());
        }
    }

    Err(UnlockError::NoKeyMatched)
}

/// Create the 16-byte unlock response from the 8-byte challenge and a 64-bit key.
/// Ported from ArgyllCMS spectro/i1d3.c create_unlock_response().
fn create_unlock_response(sc: [u8; 8], k0: u32, k1: u32) -> [u8; 16] {
    // Shuffle challenge bytes into two 32-bit integers
    let ci0 = ((sc[3] as u32) << 24)
        | ((sc[0] as u32) << 16)
        | ((sc[4] as u32) << 8)
        | (sc[6] as u32);
    let ci1 = ((sc[1] as u32) << 24)
        | ((sc[7] as u32) << 16)
        | ((sc[2] as u32) << 8)
        | (sc[5] as u32);

    // Negate keys (32-bit wrap, matching C unsigned int behavior)
    let nk0 = k0.wrapping_neg();
    let nk1 = k1.wrapping_neg();

    // Compute products/differences
    let co0 = nk0.wrapping_sub(ci1);
    let co1 = nk1.wrapping_sub(ci0);
    let co2 = ci1.wrapping_mul(nk0);
    let co3 = ci0.wrapping_mul(nk1);

    // Compute sum of challenge bytes + sum of negated key bytes
    let mut sum: u32 = 0;
    for b in sc {
        sum = sum.wrapping_add(b as u32);
    }
    for byte in nk0.to_le_bytes() {
        sum = sum.wrapping_add(byte as u32);
    }
    for byte in nk1.to_le_bytes() {
        sum = sum.wrapping_add(byte as u32);
    }

    let s0 = (sum & 0xFF) as u8;
    let s1 = ((sum >> 8) & 0xFF) as u8;

    let mut sr = [0u8; 16];
    sr[0]  = (((co0 >> 16) & 0xFF) as u8).wrapping_add(s0);
    sr[1]  = (((co2 >>  8) & 0xFF) as u8).wrapping_sub(s1);
    sr[2]  = (((co3       ) & 0xFF) as u8).wrapping_add(s1);
    sr[3]  = (((co1 >> 16) & 0xFF) as u8).wrapping_add(s0);
    sr[4]  = (((co2 >> 16) & 0xFF) as u8).wrapping_sub(s1);
    sr[5]  = (((co3 >> 16) & 0xFF) as u8).wrapping_sub(s0);
    sr[6]  = (((co1 >> 24) & 0xFF) as u8).wrapping_sub(s0);
    sr[7]  = (((co0       ) & 0xFF) as u8).wrapping_sub(s1);
    sr[8]  = (((co3 >>  8) & 0xFF) as u8).wrapping_add(s0);
    sr[9]  = (((co2 >> 24) & 0xFF) as u8).wrapping_sub(s1);
    sr[10] = (((co0 >>  8) & 0xFF) as u8).wrapping_add(s0);
    sr[11] = (((co1 >>  8) & 0xFF) as u8).wrapping_sub(s1);
    sr[12] = (((co1       ) & 0xFF) as u8).wrapping_add(s1);
    sr[13] = (((co3 >> 24) & 0xFF) as u8).wrapping_add(s1);
    sr[14] = (((co2       ) & 0xFF) as u8).wrapping_add(s0);
    sr[15] = (((co0 >> 24) & 0xFF) as u8).wrapping_sub(s0);

    sr
}

/// Parse a 16-character hex string into a (u32, u32) key pair.
fn parse_escape_key(hex: &str) -> Option<(u32, u32)> {
    let hex = hex.trim();
    if hex.len() != 16 {
        return None;
    }
    let mut bytes = [0u8; 8];
    for (i, chunk) in hex.as_bytes().chunks_exact(2).enumerate() {
        let hi = hex_digit(chunk[0])?;
        let lo = hex_digit(chunk[1])?;
        bytes[i] = (hi << 4) | lo;
    }
    let k0 = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let k1 = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    Some((k0, k1))
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_escape_key_valid() {
        let key = parse_escape_key("e9622e9f8d63e133").unwrap();
        assert_eq!(key.0, 0xe9622e9f);
        assert_eq!(key.1, 0x8d63e133);
    }

    #[test]
    fn test_parse_escape_key_invalid_length() {
        assert!(parse_escape_key("1234").is_none());
        assert!(parse_escape_key("").is_none());
    }

    #[test]
    fn test_parse_escape_key_invalid_chars() {
        assert!(parse_escape_key("gghh1122aabbccdd").is_none());
    }

    #[test]
    fn test_create_unlock_response_known_vectors() {
        let sc = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let k0 = 0xe9622e9f;
        let k1 = 0x8d63e133;

        let resp = create_unlock_response(sc, k0, k1);

        // Verify the response is deterministic (same input -> same output)
        let resp2 = create_unlock_response(sc, k0, k1);
        assert_eq!(resp, resp2);

        // Verify different key produces different output
        let resp3 = create_unlock_response(sc, 0x11111111, 0x22222222);
        assert_ne!(resp, resp3);
    }

    #[test]
    fn test_known_keys_count() {
        assert_eq!(KNOWN_KEYS.len(), 11);
    }
}
