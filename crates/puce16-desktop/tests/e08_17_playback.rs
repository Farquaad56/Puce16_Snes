use puce16_desktop::audio::{AudioRing, Playback};

fn fill_ring(ring: &mut AudioRing, frames: usize, l: i16, r: i16) {
    let data = vec![[l, r]; frames];
    ring.push(&data);
}

#[test]
fn e08_17_constant_stereo() {
    let mut ring = AudioRing::new();
    fill_ring(&mut ring, 4000, 16384, 16384);

    let mut pb = Playback::new(48000);
    let frames = 512;
    let mut out = vec![0.0f32; frames * 2];
    pb.render(&mut ring, 1.0, false, 2, &mut out);

    // After warmup (first ~16 frames), values should be near 0.5
    for i in 16..frames {
        let l = out[i * 2];
        let r = out[i * 2 + 1];
        assert!((l - 0.5).abs() < 0.01, "frame {} left: {}", i, l);
        assert!((r - 0.5).abs() < 0.01, "frame {} right: {}", i, r);
    }
}

#[test]
fn e08_17_volume_scales() {
    let mut ring = AudioRing::new();
    fill_ring(&mut ring, 4000, 16384, 16384);

    let mut pb = Playback::new(48000);
    let frames = 512;
    let mut out = vec![0.0f32; frames * 2];
    pb.render(&mut ring, 0.5, false, 2, &mut out);

    // Volume 0.5 -> values near 0.25 after warmup
    for i in 16..frames {
        let l = out[i * 2];
        let r = out[i * 2 + 1];
        assert!((l - 0.25).abs() < 0.01, "frame {} left: {}", i, l);
        assert!((r - 0.25).abs() < 0.01, "frame {} right: {}", i, r);
    }
}

#[test]
fn e08_17_muted_is_silent() {
    let mut ring = AudioRing::new();
    fill_ring(&mut ring, 4000, 16384, 16384);
    let initial_len = ring.len();

    let mut pb = Playback::new(48000);
    let frames = 512;
    let mut out = vec![0.0f32; frames * 2];
    pb.render(&mut ring, 1.0, true, 2, &mut out);

    // All values exactly 0.0 when muted
    for v in &out {
        assert_eq!(*v, 0.0f32, "muted value not zero: {}", v);
    }

    // Ring was still consumed
    assert!(ring.len() < initial_len, "ring not consumed while muted");
}

#[test]
fn e08_17_mono_downmix() {
    // Test 1: [16384, -16384] -> average is 0.0
    let mut ring = AudioRing::new();
    fill_ring(&mut ring, 4000, 16384, -16384);

    let mut pb = Playback::new(48000);
    let frames = 512;
    let mut out = vec![0.0f32; frames];
    pb.render(&mut ring, 1.0, false, 1, &mut out);

    for i in 16..frames {
        assert!((out[i] - 0.0).abs() < 0.01, "frame {} mono: {}", i, out[i]);
    }

    // Test 2: [16384, 0] -> average is 0.25
    let mut ring2 = AudioRing::new();
    fill_ring(&mut ring2, 4000, 16384, 0);

    let mut pb2 = Playback::new(48000);
    let mut out2 = vec![0.0f32; frames];
    pb2.render(&mut ring2, 1.0, false, 1, &mut out2);

    for i in 16..frames {
        assert!((out2[i] - 0.25).abs() < 0.01, "frame {} mono: {}", i, out2[i]);
    }
}

#[test]
fn e08_17_six_channels() {
    let mut ring = AudioRing::new();
    fill_ring(&mut ring, 4000, 16384, 16384);

    let mut pb = Playback::new(48000);
    let frames = 512;
    let mut out = vec![0.0f32; frames * 6];
    pb.render(&mut ring, 1.0, false, 6, &mut out);

    for i in 16..frames {
        // Channels 0 and 1 near 0.5 after warmup
        assert!((out[i * 6] - 0.5).abs() < 0.01, "frame {} ch0: {}", i, out[i * 6]);
        assert!((out[i * 6 + 1] - 0.5).abs() < 0.01, "frame {} ch1: {}", i, out[i * 6 + 1]);
        // Channels 2-5 exactly 0.0
        for ch in 2..6 {
            assert_eq!(out[i * 6 + ch], 0.0f32, "frame {} ch{} not zero", i, ch);
        }
    }
}

#[test]
fn e08_17_underrun_repeats_last() {
    let mut ring = AudioRing::new();
    fill_ring(&mut ring, 100, 16384, 16384);

    let mut pb = Playback::new(32040); // Same rate as source
    let frames = 600;
    let mut out = vec![0.0f32; frames * 2];
    pb.render(&mut ring, 1.0, false, 2, &mut out);

    // Find the last non-underrun value (should be around frame 100)
    // After underrun starts, all values should equal that last value

    // Check that the last 50 frames are all equal to each other (repeated)
    let last_50_start = frames - 50;
    for i in last_50_start..frames {
        assert_eq!(out[i * 2], out[last_50_start * 2], "underrun frame {} left differs", i);
        assert_eq!(out[i * 2 + 1], out[last_50_start * 2 + 1], "underrun frame {} right differs", i);
    }

    // And they're not zero (would indicate a jump to silence)
    assert_ne!(out[last_50_start * 2], 0.0f32, "underrun jumped to zero");
}

#[test]
fn e08_17_empty_ring_start_is_silent() {
    let mut ring = AudioRing::new(); // Empty

    let mut pb = Playback::new(48000);
    let frames = 256;
    let mut out = vec![0.0f32; frames * 2];
    pb.render(&mut ring, 1.0, false, 2, &mut out);

    // All values should be 0.0 (last initial value)
    for v in &out {
        assert_eq!(*v, 0.0f32, "empty ring produced non-zero: {}", v);
    }
}

#[test]
fn e08_17_consumption_rate() {
    let mut ring = AudioRing::new();
    fill_ring(&mut ring, 8000, 16384, 16384);

    let mut pb = Playback::new(48000);
    let frames = 4800;
    let mut out = vec![0.0f32; frames * 2];
    pb.render(&mut ring, 1.0, false, 2, &mut out);

    // Expected consumption: 4800 * (32040/48000) = 3204 frames
    let consumed = 8000 - ring.len();
    assert!(consumed >= 3200 && consumed <= 3260, "consumed {} frames (expected 3200-3260)", consumed);
}

#[test]
fn e08_17_zero_channels_noop() {
    let mut ring = AudioRing::new();
    fill_ring(&mut ring, 100, 16384, 16384);
    let initial_len = ring.len();

    let mut pb = Playback::new(48000);
    let mut out = vec![0.0f32; 100];
    // Should not panic with channels=0
    pb.render(&mut ring, 1.0, false, 0, &mut out);

    // Ring untouched
    assert_eq!(ring.len(), initial_len, "ring modified with channels=0");
}
