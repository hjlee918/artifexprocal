use hal::types::Lut1D;

pub struct Lut1DGenerator;

impl Lut1DGenerator {
    pub fn from_corrections(
        corrections: &[Vec<f64>; 3],
        lut_size: usize,
    ) -> Lut1D {
        let patch_count = corrections[0].len();
        let mut channels: [Vec<f64>; 3] = [
            Vec::with_capacity(lut_size),
            Vec::with_capacity(lut_size),
            Vec::with_capacity(lut_size),
        ];

        for i in 0..lut_size {
            let input = i as f64 / (lut_size.saturating_sub(1).max(1) as f64);
            let patch_index_f = input * (patch_count.saturating_sub(1).max(1) as f64);
            let idx_low = patch_index_f.floor() as usize;
            let idx_high = (idx_low + 1).min(patch_count.saturating_sub(1));
            let t = patch_index_f - idx_low as f64;

            for ch in 0..3 {
                let corr_low = corrections[ch].get(idx_low).copied().unwrap_or(1.0);
                let corr_high = corrections[ch].get(idx_high).copied().unwrap_or(1.0);
                let corr = corr_low + t * (corr_high - corr_low);
                let output = (input * corr).clamp(0.0, 1.0);
                channels[ch].push(output);
            }
        }

        Lut1D { channels, size: lut_size }
    }
}
