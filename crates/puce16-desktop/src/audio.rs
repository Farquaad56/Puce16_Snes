// Audio ring buffer: circular buffer between emulation thread (producer) and audio callback (consumer).

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub const RING_CAPACITY: usize = 8192; // frames
pub const RING_TARGET: usize = 2048;   // target fill level (~64 ms)
pub const RING_WAIT_ABOVE: usize = 3072; // emulation waits above this (~96 ms)

pub struct AudioRing {
    buf: Vec<[i16; 2]>,
    head: usize,
    len: usize,
}

impl AudioRing {
    pub fn new() -> AudioRing {
        AudioRing {
            buf: vec![[0; 2]; RING_CAPACITY],
            head: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        RING_CAPACITY
    }

    pub fn free(&self) -> usize {
        RING_CAPACITY - self.len
    }

    pub fn push(&mut self, frames: &[[i16; 2]]) -> usize {
        let available = self.free();
        let to_write = frames.len().min(available);
        if to_write == 0 {
            return 0;
        }

        let tail = (self.head + self.len) % RING_CAPACITY;
        // First chunk: from tail to end of buffer
        let first_chunk = RING_CAPACITY - tail;
        if to_write <= first_chunk {
            self.buf[tail..tail + to_write].copy_from_slice(&frames[..to_write]);
        } else {
            // Wrap around
            self.buf[tail..].copy_from_slice(&frames[..first_chunk]);
            let remaining = to_write - first_chunk;
            self.buf[..remaining].copy_from_slice(&frames[first_chunk..to_write]);
        }

        self.len += to_write;
        to_write
    }

    pub fn pop(&mut self, out: &mut [[i16; 2]]) -> usize {
        let available = self.len;
        let to_read = out.len().min(available);
        if to_read == 0 {
            return 0;
        }

        // First chunk: from head to end of buffer
        let first_chunk = RING_CAPACITY - self.head;
        if to_read <= first_chunk {
            out[..to_read].copy_from_slice(&self.buf[self.head..self.head + to_read]);
        } else {
            // Wrap around
            out[..first_chunk].copy_from_slice(&self.buf[self.head..]);
            let remaining = to_read - first_chunk;
            out[first_chunk..to_read].copy_from_slice(&self.buf[..remaining]);
        }

        self.head = (self.head + to_read) % RING_CAPACITY;
        self.len -= to_read;
        to_read
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }
}

pub type SharedRing = std::sync::Arc<std::sync::Mutex<AudioRing>>;

pub fn new_shared_ring() -> SharedRing {
    std::sync::Arc::new(std::sync::Mutex::new(AudioRing::new()))
}

// Sinc-windowed resampler (Hann window, 16 taps) for desktop audio.
// Converts between arbitrary sample rates using f32 arithmetic.

pub struct Resampler {
    step: f64,
    pos: f64,
    buf: Vec<[f32; 2]>,
}

impl Resampler {
    pub fn new(src_rate: u32, dst_rate: u32) -> Resampler {
        let step = src_rate as f64 / dst_rate as f64;
        // Pre-fill with 8 null frames for history (needed by interpolation kernel)
        let buf = vec![[0.0f32, 0.0f32]; 8];
        Resampler { step, pos: 8.0, buf }
    }

    pub fn process(&mut self, input: &[[f32; 2]], out: &mut Vec<[f32; 2]>) {
        // Append new input to buffer
        for frame in input {
            self.buf.push(*frame);
        }

        let buf_len = self.buf.len() as f64;

        // Generate output samples while we have enough history + lookahead
        while self.pos + 8.0 <= buf_len - 1.0 {
            let n_i64 = self.pos.floor() as i64;
            let f = self.pos - n_i64 as f64; // fractional part [0, 1)

            // Compute sinc interpolation with Hann window (16 taps: k=-7..8)
            let mut w_sum_l = 0.0f32;
            let mut w_sum_r = 0.0f32;
            let mut out_l = 0.0f32;
            let mut out_r = 0.0f32;

            for k in -7isize..=8 {
                let x = f - k as f64;
                let w = self.sinc_hann(x);
                if w != 0.0 {
                    let idx = (n_i64 + k as i64) as usize;
                    // Safe: pos+8 <= buf_len-1 ensures n+k is in range [n-7, n+8]
                    out_l += self.buf[idx][0] * w as f32;
                    out_r += self.buf[idx][1] * w as f32;
                    w_sum_l += w as f32;
                    w_sum_r += w as f32;
                }
            }

            // Normalize by weight sum (gain = 1.0 for DC)
            out.push([out_l / w_sum_l, out_r / w_sum_r]);

            self.pos += self.step;
        }

        // Trim buffer: keep from floor(pos) - 7 onwards to maintain history
        let trim_to = ((self.pos.floor() as i64) - 7).max(0) as usize;
        if trim_to > 0 {
            let shift = self.pos - trim_to as f64;
            self.buf.drain(..trim_to);
            self.pos = shift;
        }
    }

    fn sinc_hann(&self, x: f64) -> f64 {
        // Hann window: 0.5 * (1 + cos(pi * x / 8)) for |x| < 8, else 0
        let abs_x = x.abs();
        if abs_x >= 8.0 {
            return 0.0;
        }

        // Sinc function: sin(pi*x) / (pi*x), with limit 1 at x=0
        let sinc_val = if abs_x < 1e-12 {
            1.0
        } else {
            std::f64::consts::PI * x
        };

        // Windowed sinc
        let hann = 0.5 * (1.0 + (std::f64::consts::PI * x / 8.0).cos());
        if abs_x < 1e-12 {
            hann
        } else {
            ((sinc_val.sin()) / sinc_val) * hann
        }
    }
}

// Playback: ring consumer that resamples, applies volume/mute, and writes device channels.

pub const SRC_RATE: u32 = 32_040;

pub struct Playback {
    rs: Resampler,
    pending: std::collections::VecDeque<[f32; 2]>,
    last: [f32; 2],
    scratch: Vec<[i16; 2]>,
    device_rate: u32,
}

impl Playback {
    pub fn new(device_rate: u32) -> Playback {
        Playback {
            rs: Resampler::new(SRC_RATE, device_rate),
            pending: std::collections::VecDeque::new(),
            last: [0.0f32, 0.0f32],
            scratch: Vec::with_capacity(16),
            device_rate,
        }
    }

    pub fn render(&mut self, ring: &mut AudioRing, volume: f32, muted: bool, channels: usize, out: &mut [f32]) {
        if channels == 0 || out.is_empty() {
            return;
        }

        let frames = out.len() / channels;
        let mut written = 0;

        while written < frames {
            // First use the frames left over from the previous call / iteration: dropping
            // them would punch a hole in the signal (audible crackle).
            while written < frames {
                match self.pending.pop_front() {
                    Some(frame) => {
                        self.last = frame;
                        let l = if muted { 0.0f32 } else { frame[0] * volume };
                        let r = if muted { 0.0f32 } else { frame[1] * volume };
                        match channels {
                            1 => out[written] = (l + r) / 2.0,
                            2 => {
                                out[written * 2] = l;
                                out[written * 2 + 1] = r;
                            }
                            _ => {
                                out[written * channels] = l;
                                out[written * channels + 1] = r;
                            }
                        }
                        written += 1;
                    }
                    None => break,
                }
            }
            if written >= frames {
                break;
            }
            // Estimate how many source frames we need for remaining output
            let needed = frames - written;
            let src_needed = ((needed as u32 * SRC_RATE) / self.device_rate).max(16);
            self.scratch.clear();
            self.scratch.resize(src_needed as usize, [0; 2]);

            // Read from ring
            let read = ring.pop(&mut self.scratch[..src_needed as usize]);
            if read == 0 {
                // Underrun: fill remaining output with last sample
                for i in written..frames {
                    let l = if muted { 0.0f32 } else { self.last[0] * volume };
                    let r = if muted { 0.0f32 } else { self.last[1] * volume };

                    match channels {
                        1 => out[i] = (l + r) / 2.0,
                        2 => {
                            out[i * 2] = l;
                            out[i * 2 + 1] = r;
                        }
                        _ if channels > 2 => {
                            out[i * channels] = l;
                            out[i * channels + 1] = r;
                        }
                        _ => {}
                    }
                }
                break;
            }

            // Convert i16 -> f32 normalized to [-1, 1]
            let mut input_f32 = Vec::with_capacity(read);
            for frame in &self.scratch[..read] {
                input_f32.push([frame[0] as f32 / 32768.0, frame[1] as f32 / 32768.0]);
            }

            // Resample into pending buffer
            let mut resampled = Vec::new();
            self.rs.process(&input_f32, &mut resampled);

            // Queue the resampled frames; the top of the loop writes them out.
            self.pending.extend(resampled);
        }
    }
}

// AudioOut: opens default output device via cpal and connects Playback::render to its callback.

pub struct AudioOut {
    _stream: cpal::Stream,
    volume: std::sync::Arc<std::sync::atomic::AtomicU32>,
    muted: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub device_rate: u32,
    pub channels: usize,
}

impl AudioOut {
    pub fn new(ring: SharedRing) -> Result<AudioOut, String> {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => return Err("no audio output device".to_string()),
        };

        let config = match device.default_output_config() {
            Ok(c) => c,
            Err(e) => return Err(format!("default output config error: {}", e)),
        };

        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let format = config.sample_format();

        let volume = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(1.0f32.to_bits()));
        let muted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let stream = match format {
            cpal::SampleFormat::F32 => {
                let ring_cb = ring.clone();
                let vol_cb = volume.clone();
                let mut_cb = muted.clone();
                // One Playback per stream: its resampler history must survive between callbacks.
                let mut playback = Playback::new(sample_rate);
                device.build_output_stream(
                    config.into(),
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let frames = data.len() / channels;
                        if frames == 0 {
                            return;
                        }
                        let vol_bits = vol_cb.load(std::sync::atomic::Ordering::Relaxed);
                        let vol = f32::from_bits(vol_bits);
                        let mute = mut_cb.load(std::sync::atomic::Ordering::Relaxed);
                        let mut ring_guard = match ring_cb.lock() {
                            Ok(g) => g,
                            Err(poisoned) => poisoned.into_inner(),
                        };
                        playback.render(&mut *ring_guard, vol, mute, channels, data);
                    },
                    move |err| eprintln!("audio stream error: {}", err),
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let ring_cb = ring.clone();
                let vol_cb = volume.clone();
                let mut_cb = muted.clone();
                // One Playback per stream: its resampler history must survive between callbacks.
                let mut playback = Playback::new(sample_rate);
                device.build_output_stream(
                    config.into(),
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        let frames = data.len() / channels;
                        if frames == 0 {
                            return;
                        }
                        let mut f32_buf = vec![0.0f32; frames * channels];
                        let vol_bits = vol_cb.load(std::sync::atomic::Ordering::Relaxed);
                        let vol = f32::from_bits(vol_bits);
                        let mute = mut_cb.load(std::sync::atomic::Ordering::Relaxed);
                        let mut ring_guard = match ring_cb.lock() {
                            Ok(g) => g,
                            Err(poisoned) => poisoned.into_inner(),
                        };
                        playback.render(&mut *ring_guard, vol, mute, channels, &mut f32_buf[..]);
                        for i in 0..frames {
                            let l = (f32_buf[i * channels] * 32767.0) as i16;
                            let r = if channels > 1 {
                                (f32_buf[i * channels + 1] * 32767.0) as i16
                            } else {
                                l
                            };
                            data[i * channels] = l;
                            if channels > 1 {
                                data[i * channels + 1] = r;
                            }
                        }
                    },
                    move |err| eprintln!("audio stream error: {}", err),
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let ring_cb = ring.clone();
                let vol_cb = volume.clone();
                let mut_cb = muted.clone();
                // One Playback per stream: its resampler history must survive between callbacks.
                let mut playback = Playback::new(sample_rate);
                device.build_output_stream(
                    config.into(),
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        let frames = data.len() / channels;
                        if frames == 0 {
                            return;
                        }
                        let mut f32_buf = vec![0.0f32; frames * channels];
                        let vol_bits = vol_cb.load(std::sync::atomic::Ordering::Relaxed);
                        let vol = f32::from_bits(vol_bits);
                        let mute = mut_cb.load(std::sync::atomic::Ordering::Relaxed);
                        let mut ring_guard = match ring_cb.lock() {
                            Ok(g) => g,
                            Err(poisoned) => poisoned.into_inner(),
                        };
                        playback.render(&mut *ring_guard, vol, mute, channels, &mut f32_buf[..]);
                        for i in 0..frames {
                            let l = (f32_buf[i * channels] * 32767.0 + 32768.0) as u16;
                            let r = if channels > 1 {
                                (f32_buf[i * channels + 1] * 32767.0 + 32768.0) as u16
                            } else {
                                l
                            };
                            data[i * channels] = l;
                            if channels > 1 {
                                data[i * channels + 1] = r;
                            }
                        }
                    },
                    move |err| eprintln!("audio stream error: {}", err),
                    None,
                )
            }
            _ => return Err(format!("unsupported sample format: {:?}", format)),
        };

        let stream = match stream {
            Ok(s) => s,
            Err(e) => return Err(format!("failed to build audio stream: {}", e)),
        };

        if let Err(e) = stream.play() {
            return Err(format!("failed to play audio stream: {}", e));
        }

        Ok(AudioOut {
            _stream: stream,
            volume,
            muted,
            device_rate: sample_rate,
            channels,
        })
    }

    pub fn set_volume(&self, v: f32) {
        self.volume.store(v.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_muted(&self, m: bool) {
        self.muted.store(m, std::sync::atomic::Ordering::Relaxed);
    }
}
