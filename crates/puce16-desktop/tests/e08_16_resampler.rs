use puce16_desktop::audio::Resampler;

fn make_sine(freq: f32, rate: u32, n_frames: usize) -> Vec<[f32; 2]> {
    let mut frames = Vec::with_capacity(n_frames);
    for i in 0..n_frames {
        let t = (i as f32) / (rate as f32);
        let val = (2.0 * std::f32::consts::PI * freq * t).sin() * 0.5;
        frames.push([val, -val]);
    }
    frames
}

#[test]
fn e08_16_identity_rate() {
    let mut res = Resampler::new(32040, 32040);
    let input: Vec<[f32; 2]> = (0..100)
        .map(|i| [i as f32 / 1000.0, -(i as f32) / 1000.0])
        .collect();

    let mut out = Vec::new();
    res.process(&input, &mut out);

    assert_eq!(out.len(), 92);
    for i in 0..92 {
        let diff_l = (out[i][0] - input[i][0]).abs();
        let diff_r = (out[i][1] - input[i][1]).abs();
        assert!(diff_l < 1e-4, "ch L at {}: {} vs {}", i, out[i][0], input[i][0]);
        assert!(diff_r < 1e-4, "ch R at {}: {} vs {}", i, out[i][1], input[i][1]);
    }
}

#[test]
fn e08_16_dc_is_preserved_upsampling() {
    let mut res = Resampler::new(32040, 48000);
    let input: Vec<[f32; 2]> = vec![[0.5, 0.5]; 1000];

    let mut out = Vec::new();
    res.process(&input, &mut out);

    assert!(out.len() >= 1480 && out.len() <= 1500, "len={}", out.len());
    for i in 16..out.len() {
        let diff_l = (out[i][0] - 0.5).abs();
        let diff_r = (out[i][1] - 0.5).abs();
        assert!(diff_l < 1e-4, "ch L at {}: {}", i, out[i][0]);
        assert!(diff_r < 1e-4, "ch R at {}: {}", i, out[i][1]);
    }
}

#[test]
fn e08_16_dc_preserved_downsampling() {
    let mut res = Resampler::new(48000, 32040);
    let input: Vec<[f32; 2]> = vec![[0.25, -0.25]; 1000];

    let mut out = Vec::new();
    res.process(&input, &mut out);

    assert!(out.len() >= 660 && out.len() <= 680, "len={}", out.len());
    for i in 16..out.len() {
        let diff_l = (out[i][0] - 0.25).abs();
        let diff_r = (out[i][1] + 0.25).abs();
        assert!(diff_l < 1e-4, "ch L at {}: {}", i, out[i][0]);
        assert!(diff_r < 1e-4, "ch R at {}: {}", i, out[i][1]);
    }
}

#[test]
fn e08_16_streaming_equals_one_shot() {
    let input = make_sine(440.0, 32040, 2000);

    // One-shot processing
    let mut res1 = Resampler::new(32040, 44100);
    let mut out1 = Vec::new();
    res1.process(&input, &mut out1);

    // Streaming in chunks of varying sizes (7, 100, 1 cycling)
    let mut res2 = Resampler::new(32040, 44100);
    let mut out2 = Vec::new();
    let chunk_sizes = [7, 100, 1];
    let mut pos = 0;
    let mut idx = 0;
    while pos < input.len() {
        let size = chunk_sizes[idx % chunk_sizes.len()];
        let end = (pos + size).min(input.len());
        res2.process(&input[pos..end], &mut out2);
        pos = end;
        idx += 1;
    }

    assert_eq!(out1.len(), out2.len(), "lengths differ: {} vs {}", out1.len(), out2.len());
    let mut max_diff = 0.0f32;
    for i in 0..out1.len() {
        let d_l = (out1[i][0] - out2[i][0]).abs();
        let d_r = (out1[i][1] - out2[i][1]).abs();
        max_diff = max_diff.max(d_l).max(d_r);
    }
    assert!(max_diff < 1e-6, "max diff={}", max_diff);
}

#[test]
fn e08_16_sine_amplitude() {
    let input = make_sine(1000.0, 32040, 4000);

    let mut res = Resampler::new(32040, 48000);
    let mut out = Vec::new();
    res.process(&input, &mut out);

    // Find max amplitude after first 100 samples (transient)
    let mut max_amp = 0.0f32;
    for i in 100..out.len() {
        max_amp = max_amp.max(out[i][0].abs()).max(out[i][1].abs());
    }
    assert!(max_amp >= 0.49 && max_amp <= 0.51, "amplitude={}", max_amp);
}

#[test]
fn e08_16_empty_input() {
    let mut res = Resampler::new(32040, 48000);
    let mut out = Vec::new();
    res.process(&[], &mut out);
    assert_eq!(out.len(), 0);
}

#[test]
fn e08_16_output_rate() {
    let input: Vec<[f32; 2]> = vec![[0.5, -0.5]; 32040];

    let mut res = Resampler::new(32040, 48000);
    let mut out = Vec::new();
    res.process(&input, &mut out);

    assert!(out.len() >= 47975 && out.len() <= 48000, "len={}", out.len());
}

#[test]
fn e08_16_channels_independent() {
    let input: Vec<[f32; 2]> = vec![[0.5, 0.0]; 1000];

    let mut res = Resampler::new(32040, 48000);
    let mut out = Vec::new();
    res.process(&input, &mut out);

    for i in 16..out.len() {
        let diff_l = (out[i][0] - 0.5).abs();
        let diff_r = out[i][1].abs();
        assert!(diff_l < 1e-4, "ch L at {}: {}", i, out[i][0]);
        assert!(diff_r < 1e-4, "ch R at {}: {}", i, out[i][1]);
    }
}
