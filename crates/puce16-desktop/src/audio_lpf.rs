//! SNES output low-pass filter, after the MiSTer audio filter
//! `Filters_Audio/Core Specific/SNES GPM-02 LPF.txt` (https://github.com/MiSTer-devel/Filters_MiSTer).
//!
//! The MiSTer file describes a 3rd-order IIR filter running at 6.144 MHz. Its poles correspond to
//! an analog all-pole low-pass: one real pole at 5.50 kHz and a complex pair (|p| = 14.30 kHz).
//! Those analog poles (derived from the file's denominator by the inverse bilinear transform)
//! are re-discretised here at the actual output sample rate with the impulse-invariant method,
//! and the gain is normalised to 0 dB at DC (the file's own base gain is about +21 dB).

/// Analog pole, real part of the real pole (rad/s).
const P_REAL: f64 = -34557.61026847;
/// Analog complex pole pair: sigma +/- j*omega (rad/s).
const P_SIG: f64 = -43783.78785255;
const P_OMG: f64 = 78445.24783766;

#[derive(Clone, Copy)]
struct C {
    re: f64,
    im: f64,
}

impl C {
    fn new(re: f64, im: f64) -> C {
        C { re, im }
    }
    fn sub(self, o: C) -> C {
        C::new(self.re - o.re, self.im - o.im)
    }
    fn mul(self, o: C) -> C {
        C::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    fn div(self, o: C) -> C {
        let d = o.re * o.re + o.im * o.im;
        C::new((self.re * o.re + self.im * o.im) / d, (self.im * o.re - self.re * o.im) / d)
    }
    fn conj(self) -> C {
        C::new(self.re, -self.im)
    }
    fn exp(self) -> C {
        let m = self.re.exp();
        C::new(m * self.im.cos(), m * self.im.sin())
    }
    fn scale(self, k: f64) -> C {
        C::new(self.re * k, self.im * k)
    }
}

/// Filter coefficients: a one-pole section (`r_*`) and a biquad section (`p_*`) in parallel.
#[derive(Clone, Copy, Debug)]
pub struct Coefs {
    r_b0: f64,
    r_a1: f64,
    p_b0: f64,
    p_b1: f64,
    p_a1: f64,
    p_a2: f64,
}

/// Design the filter for an output sample rate of `fs` Hz.
pub fn design(fs: f64) -> Coefs {
    let t = 1.0 / fs;
    let p0 = C::new(P_REAL, 0.0);
    let pc = C::new(P_SIG, P_OMG);
    let pcc = pc.conj();
    // Residues of G(s) = prod(-p) / prod(s - p).
    let g0 = C::new(-P_REAL * (P_SIG * P_SIG + P_OMG * P_OMG), 0.0);
    let r0 = g0.div(p0.sub(pc).mul(p0.sub(pcc)));
    let r1 = g0.div(pc.sub(p0).mul(pc.sub(pcc)));
    // Impulse invariance: H(z) = sum T*r / (1 - exp(pT) z^-1).
    let l0 = p0.scale(t).exp();
    let l1 = pc.scale(t).exp();
    let c0 = r0.scale(t);
    let c1 = r1.scale(t);
    // DC gain, to normalise to unity.
    let one = C::new(1.0, 0.0);
    let dc = c0.div(one.sub(l0)).re + 2.0 * c1.div(one.sub(l1)).re;
    let (c0, c1) = (c0.scale(1.0 / dc), c1.scale(1.0 / dc));
    Coefs {
        r_b0: c0.re,
        r_a1: -l0.re,
        // 2 Re( c / (1 - l z^-1) ) = (2 Re c - 2 Re(c conj(l)) z^-1) / (1 - 2 Re l z^-1 + |l|^2 z^-2)
        p_b0: 2.0 * c1.re,
        p_b1: -2.0 * c1.mul(l1.conj()).re,
        p_a1: -2.0 * l1.re,
        p_a2: l1.re * l1.re + l1.im * l1.im,
    }
}

/// Stereo filter state.
#[derive(Clone, Debug)]
pub struct Gpm02Lpf {
    coefs: Coefs,
    r_y: [f64; 2],
    s1: [f64; 2],
    s2: [f64; 2],
}

impl Gpm02Lpf {
    pub fn new(sample_rate: u32) -> Gpm02Lpf {
        Gpm02Lpf { coefs: design(sample_rate.max(1) as f64), r_y: [0.0; 2], s1: [0.0; 2], s2: [0.0; 2] }
    }

    /// Clear the filter memory.
    pub fn reset(&mut self) {
        self.r_y = [0.0; 2];
        self.s1 = [0.0; 2];
        self.s2 = [0.0; 2];
    }

    /// Filter one stereo frame.
    pub fn process(&mut self, frame: [f32; 2]) -> [f32; 2] {
        let c = self.coefs;
        let mut out = [0.0f32; 2];
        for ch in 0..2 {
            let x = frame[ch] as f64;
            // One-pole section: y = b0 x - a1 y[n-1]
            let yr = c.r_b0 * x - c.r_a1 * self.r_y[ch];
            self.r_y[ch] = yr;
            // Biquad section, transposed direct form II.
            let yp = c.p_b0 * x + self.s1[ch];
            self.s1[ch] = c.p_b1 * x - c.p_a1 * yp + self.s2[ch];
            self.s2[ch] = -c.p_a2 * yp;
            out[ch] = (yr + yp) as f32;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Steady-state gain in dB of a sine at `freq` through the filter at `rate`.
    fn gain_db(rate: u32, freq: f64) -> f64 {
        let mut f = Gpm02Lpf::new(rate);
        let n = rate as usize; // one second
        let mut peak = 0.0f64;
        for i in 0..n {
            let x = (2.0 * std::f64::consts::PI * freq * i as f64 / rate as f64).sin() as f32;
            let y = f.process([x, x])[0] as f64;
            if i > n / 2 {
                peak = peak.max(y.abs());
            }
        }
        20.0 * peak.log10()
    }

    #[test]
    fn unity_gain_at_dc() {
        let mut f = Gpm02Lpf::new(48000);
        let mut y = [0.0f32; 2];
        for _ in 0..5000 {
            y = f.process([1.0, -1.0]);
        }
        assert!((y[0] - 1.0).abs() < 1e-3, "{}", y[0]);
        assert!((y[1] + 1.0).abs() < 1e-3, "{}", y[1]);
    }

    #[test]
    fn response_matches_the_analog_prototype() {
        // Analog prototype: -0.12 dB @ 1 kHz, -2.1 dB @ 5 kHz, -4.9 dB @ 10 kHz, -9.5 dB @ 15 kHz.
        for (freq, expected) in [(1000.0, -0.12), (5000.0, -2.09), (10000.0, -4.95), (15000.0, -9.5)] {
            let g = gain_db(48000, freq);
            assert!((g - expected).abs() < 1.0, "{freq} Hz: {g} dB, expected {expected}");
        }
    }

    #[test]
    fn channels_are_independent() {
        let mut f = Gpm02Lpf::new(44100);
        let mut y = [0.0f32; 2];
        for _ in 0..3000 {
            y = f.process([0.5, 0.0]);
        }
        assert!((y[0] - 0.5).abs() < 1e-3);
        assert!(y[1].abs() < 1e-6);
    }
}
