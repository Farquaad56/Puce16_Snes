//! MiSTer display presets (https://github.com/MiSTer-devel/Presets_MiSTer, `Display Specific/`):
//! "Sony Trinitron" and "Faded Tube". A preset combines
//!
//! 1. a horizontal 4-tap polyphase filter (`hfilter`, 64 phases),
//! 2. an adaptive vertical scanline filter (`vfilter`, 256 phases): a dark and a bright
//!    coefficient set blended per pixel and per colour channel by the brightness of the source,
//! 3. for the Trinitron, an optional aperture grille (vertical R/G/B stripes, 1:1 display),
//! 4. a gamma curve (`gamma`, `Out = 255 * (In / 255) ^ g`).
//!
//! The pipeline runs on the CPU on the upscaled image (needs an integer scale >= 2).
//! The "Faded Tube" preset names a mask file that no longer exists in the ShadowMasks_MiSTer
//! repository; as on MiSTer (missing file = option disabled) it is applied without a mask.

use crate::filters_data::Coefs;
use crate::presets_data::*;

/// Available display presets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Preset {
    None,
    SonyTrinitron,
    FadedTube,
}

type Phases256 = [[i32; 4]; 256];

impl Preset {
    pub fn from_name(s: &str) -> Preset {
        match s {
            "trinitron" => Preset::SonyTrinitron,
            "faded_tube" => Preset::FadedTube,
            _ => Preset::None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Preset::None => "none",
            Preset::SonyTrinitron => "trinitron",
            Preset::FadedTube => "faded_tube",
        }
    }

    /// Horizontal filter.
    fn hfilter(self) -> Option<&'static Coefs> {
        match self {
            Preset::None => None,
            Preset::SonyTrinitron => Some(&GS_035),
            Preset::FadedTube => Some(&GS_015),
        }
    }

    /// Adaptive vertical filter: (dark set, bright set).
    fn vfilter(self) -> Option<(&'static Phases256, &'static Phases256)> {
        match self {
            Preset::None => None,
            Preset::SonyTrinitron => Some((&SLA_DK040_BR080_DARK, &SLA_DK040_BR080_BRIGHT)),
            Preset::FadedTube => Some((&SLA_DK020_BR060_DARK, &SLA_DK020_BR060_BRIGHT)),
        }
    }

    /// Gamma exponent (`gamma_110` = 1.10, `gamma_145` = 1.45).
    fn gamma(self) -> Option<f64> {
        match self {
            Preset::None => None,
            Preset::SonyTrinitron => Some(1.10),
            Preset::FadedTube => Some(1.45),
        }
    }
}

/// Gain applied before the grille (x256): 1.4.
const GRILLE_GAIN: u32 = 358;
/// Light leaking onto the two other stripes of a triad (x256): about 8 %.
const GRILLE_LEAK: u32 = 20;

/// 256-entry gamma lookup table.
pub fn gamma_lut(g: f64) -> [u8; 256] {
    let mut lut = [0u8; 256];
    for (i, v) in lut.iter_mut().enumerate() {
        *v = (255.0 * (i as f64 / 255.0).powf(g)).round().clamp(0.0, 255.0) as u8;
    }
    lut
}

/// Apply a preset: scale by `scale` in both axes with the preset's filters, then mask and gamma.
/// Returns the pixels and the new width and height. With `Preset::None` or `scale < 2` the
/// image is returned unchanged.
pub fn apply_preset(rgba: &[u8], width: usize, scale: usize, preset: Preset, use_mask: bool) -> (Vec<u8>, usize, usize) {
    if width == 0 || rgba.is_empty() {
        return (rgba.to_vec(), width, 0);
    }
    let height = rgba.len() / (width * 4);
    let (h_table, (dark, bright)) = match (preset.hfilter(), preset.vfilter()) {
        (Some(h), Some(v)) if scale >= 2 => (h, v),
        _ => return (rgba.to_vec(), width, height),
    };

    // 1. Horizontal polyphase scaling.
    let (mid, mid_w, _) = crate::video::filter_scale(rgba, width, scale, Some(h_table), None);

    // 2. Adaptive vertical scanlines (256 phases).
    let out_h = height * scale;
    let row_len = mid_w * 4;
    let mut out = Vec::with_capacity(out_h * row_len);
    let den = 2 * scale as i64;
    for y in 0..out_h {
        let num = (2 * y + 1) as i64 - scale as i64;
        let i = num.div_euclid(den);
        let phase = ((num.rem_euclid(den) * 256) / den).clamp(0, 255) as usize;
        let (cd, cb) = (dark[phase], bright[phase]);
        let line = |t: i64| {
            let idx = (i - 1 + t).clamp(0, height as i64 - 1) as usize;
            &mid[idx * row_len..(idx + 1) * row_len]
        };
        let (l0, l1, l2, l3) = (line(0), line(1), line(2), line(3));
        for x in 0..mid_w {
            let p = x * 4;
            for c in 0..3 {
                let v = [l0[p + c] as i32, l1[p + c] as i32, l2[p + c] as i32, l3[p + c] as i32];
                // Brightness of the two source lines around this output row decides the blend.
                let t = v[1].max(v[2]);
                let ds = cd[0] * v[0] + cd[1] * v[1] + cd[2] * v[2] + cd[3] * v[3];
                let bs = cb[0] * v[0] + cb[1] * v[1] + cb[2] * v[2] + cb[3] * v[3];
                let s = (ds * (255 - t) + bs * t + 255 * 128) / (255 * 256);
                out.push(s.clamp(0, 255) as u8);
            }
            out.push(255);
        }
    }

    // 3. Aperture grille (Sony Trinitron): vertical R, G, B phosphor stripes, one output
    //    pixel wide each (period 3). Meant to be shown 1:1 on screen pixels. The signal is
    //    boosted first to compensate for the light lost between stripes.
    if use_mask && preset == Preset::SonyTrinitron {
        for px in out.chunks_exact_mut(4).enumerate().map(|(i, p)| (i % mid_w, p)) {
            let (x, p) = px;
            for c in 0..3 {
                let boosted = (p[c] as u32 * GRILLE_GAIN) >> 8;
                let f = if x % 3 == c { 256 } else { GRILLE_LEAK };
                p[c] = ((boosted.min(255) * f) >> 8) as u8;
            }
        }
    }

    // 4. Gamma.
    if let Some(g) = preset.gamma() {
        let lut = gamma_lut(g);
        for px in out.chunks_exact_mut(4) {
            px[0] = lut[px[0] as usize];
            px[1] = lut[px[1] as usize];
            px[2] = lut[px[2] as usize];
        }
    }
    (out, mid_w, out_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(w: usize, h: usize, v: u8) -> Vec<u8> {
        let mut img = Vec::new();
        for _ in 0..w * h {
            img.extend_from_slice(&[v, v, v, 255]);
        }
        img
    }

    #[test]
    fn names_round_trip() {
        for p in [Preset::None, Preset::SonyTrinitron, Preset::FadedTube] {
            assert_eq!(Preset::from_name(p.name()), p);
        }
        assert_eq!(Preset::from_name("zzz"), Preset::None);
    }

    #[test]
    fn gamma_lut_matches_the_mister_files() {
        let l = gamma_lut(1.10);
        assert_eq!(&l[..5], &[0, 1, 1, 2, 3]); // gamma_110.txt starts 0,1,1,2,3
        assert_eq!(l[255], 255);
        assert_eq!(gamma_lut(1.45)[0], 0);
    }

    #[test]
    fn sizes_and_identity_cases() {
        let img = flat(4, 3, 200);
        let (out, w, h) = apply_preset(&img, 4, 3, Preset::SonyTrinitron, true);
        assert_eq!((w, h), (12, 9));
        assert_eq!(out.len(), 12 * 9 * 4);
        assert_eq!(apply_preset(&img, 4, 1, Preset::SonyTrinitron, true), (img.clone(), 4, 3));
        assert_eq!(apply_preset(&img, 4, 3, Preset::None, true), (img.clone(), 4, 3));
    }

    #[test]
    fn scanline_gap_is_deeper_on_dark_content_than_on_bright() {
        // Faded Tube: gap ~24 % on dark pixels, ~60 % on bright ones (relative to the line centre).
        let ratio = |v: u8| {
            let (out, w, _) = apply_preset(&flat(6, 6, v), 6, 4, Preset::FadedTube, false);
            let row = |y: usize| out[(y * w + 8) * 4] as f64; // a column away from the edges
            // rows 12..16 are source line 3: centre rows 13/14, boundary with the next line at 15/16
            row(15).min(row(16)) / row(13).max(row(14))
        };
        let dark = ratio(60);
        let bright = ratio(240);
        assert!(dark < bright, "dark {dark} bright {bright}");
    }

    #[test]
    fn grille_lights_one_channel_per_column() {
        let (out, w, h) = apply_preset(&flat(8, 8, 200), 8, 3, Preset::SonyTrinitron, true);
        assert_eq!(h, 24);
        let px = |x: usize, y: usize, c: usize| out[(y * w + x) * 4 + c] as i32;
        for c in 0..3 {
            // column x with x % 3 == c is the bright stripe of channel c
            let x = 9 + c;
            assert!(px(x, 13, c) > px(x, 13, (c + 1) % 3) + 50, "c={c}");
        }
        // without the grille all channels of a flat grey are equal
        let (flat_out, w2, _) = apply_preset(&flat(8, 8, 200), 8, 3, Preset::SonyTrinitron, false);
        assert_eq!(flat_out[(13 * w2 + 9) * 4], flat_out[(13 * w2 + 9) * 4 + 1]);
    }
}
