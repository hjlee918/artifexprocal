/// sRGB gamma encoding (linear to sRGB)
pub fn srgb_gamma_encode(linear: f64) -> f64 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB gamma decoding (sRGB to linear)
pub fn srgb_gamma_decode(encoded: f64) -> f64 {
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

/// Pure gamma power law encoding
pub fn gamma_encode(linear: f64, gamma: f64) -> f64 {
    linear.powf(1.0 / gamma)
}

/// Pure gamma power law decoding
pub fn gamma_decode(encoded: f64, gamma: f64) -> f64 {
    encoded.powf(gamma)
}

/// BT.1886 gamma curve (display gamma ~2.4 with black level compensation)
pub fn bt1886_encode(linear: f64, black: f64, white: f64) -> f64 {
    let gamma = 2.4;
    let a = (white.powf(1.0 / gamma) - black.powf(1.0 / gamma)).powf(gamma);
    let b = black.powf(1.0 / gamma) / (white.powf(1.0 / gamma) - black.powf(1.0 / gamma));
    a * (linear + b).powf(gamma)
}

/// Perceptual Quantizer (PQ) / SMPTE ST.2084 encoding
/// Input: normalized linear luminance (0.0 to 1.0 = 0 to 10000 nits)
pub fn pq_encode(linear: f64) -> f64 {
    const M1: f64 = 2610.0 / 4096.0 * (1.0 / 4.0);
    const M2: f64 = 2523.0 / 4096.0 * 128.0;
    const C1: f64 = 3424.0 / 4096.0;
    const C2: f64 = 2413.0 / 4096.0 * 32.0;
    const C3: f64 = 2392.0 / 4096.0 * 32.0;

    let l = linear.abs().clamp(0.0, 1.0);
    let lm = l.powf(M1);
    let num = C1 + C2 * lm;
    let den = 1.0 + C3 * lm;

    (num / den).powf(M2)
}

/// PQ decoding
pub fn pq_decode(encoded: f64) -> f64 {
    const M1: f64 = 2610.0 / 4096.0 * (1.0 / 4.0);
    const M2: f64 = 2523.0 / 4096.0 * 128.0;
    const C1: f64 = 3424.0 / 4096.0;
    const C2: f64 = 2413.0 / 4096.0 * 32.0;
    const C3: f64 = 2392.0 / 4096.0 * 32.0;

    let n = encoded.abs().clamp(0.0, 1.0);
    let nd = n.powf(1.0 / M2);
    let num = (nd - C1).max(0.0);
    let den = C2 - C3 * nd;

    if den <= 0.0 {
        0.0
    } else {
        (num / den).powf(1.0 / M1)
    }
}

/// Hybrid Log-Gamma (HLG) encoding (BBC/NHK, ITU-R BT.2100)
/// Input: normalized linear (0.0 to 1.0)
pub fn hlg_encode(linear: f64) -> f64 {
    const A: f64 = 0.17883277;
    const B: f64 = 0.28466892;
    const C: f64 = 0.55991073;

    let l = linear.abs().clamp(0.0, 1.0);
    if l <= 1.0 / 12.0 {
        (3.0 * l).sqrt()
    } else {
        A * (12.0 * l - B).ln() + C
    }
}

/// HLG decoding
pub fn hlg_decode(encoded: f64) -> f64 {
    const A: f64 = 0.17883277;
    const B: f64 = 0.28466892;
    const C: f64 = 0.55991073;

    let e = encoded.abs().clamp(0.0, 1.0);
    if e <= 0.5 {
        e * e / 3.0
    } else {
        (((e - C) / A).exp() + B) / 12.0
    }
}
