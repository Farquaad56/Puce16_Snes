//! E06-49: DSP integration tests - constant BRR sample played in a loop.

use puce16_core::apu::Apu;
use puce16_core::hash::Fnv64;

fn setup(apu: &mut Apu, kon: u8, flg: u8) {
    // Directory at 0x0200: start=loop=0x0300
    apu.debug_poke(0x0200, &[0x00, 0x03, 0x00, 0x03]);
    // BRR block at 0x0300: shift 12, filter 0, loop+end flags, all nibbles 7 (constant)
    apu.debug_poke(0x0300, &[0xC3, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77]);
    // Infinite loop program at 0x0400: BRA -2
    apu.debug_poke(0x0400, &[0x2F, 0xFE]);
    apu.debug_set_pc(0x0400);

    // DSP register setup via debug writes
    let pairs = [
        (0x5D, 0x02), // DIR: directory at 0x0200
        (0x04, 0x00), // SRCN: sample channel 0
        (0x02, 0x00), // PITCHL
        (0x03, 0x10), // PITCHH: pitch = 0x1000 (default)
        (0x05, 0x00), // ADSR1: direct gain, no envelope
        (0x07, 0x7F), // GAIN: max -> env value 2032
        (0x00, 0x7F), // VOLL: full left volume
        (0x01, 0x7F), // VOLR: full right volume
        (0x0C, 0x7F), // MVOLL: master left volume
        (0x1C, 0x7F), // MVOLR: master right volume
        (0x6C, flg),  // FLG: flags
    ];
    for &(r, v) in &pairs {
        apu.debug_dsp_write(r, v);
    }
    apu.debug_dsp_write(0x4C, kon); // KON: key on/off
}

#[test]
fn e06_49_constant_tone() {
    let mut apu = Apu::new();
    setup(&mut apu, 1, 0x20);
    apu.step_to(1_000_000);
    let v = apu.take_audio();

    assert!(v.len() % 2 == 0, "audio length must be even");
    assert!(v.len() >= 2 * 1440 && v.len() <= 2 * 1520,
        "expected 2880..=3040 samples, got {}", v.len());

    // After initial transient (first 40 pairs), output should be stable
    for i in (2 * 40)..v.len() {
        assert!(v[i] >= 27000 && v[i] <= 29000,
            "sample {} out of range: {}", i, v[i]);
    }

    // Left and right channels should be identical for each pair
    for k in 0..(v.len() / 2) {
        assert_eq!(v[2 * k], v[2 * k + 1],
            "channels differ at pair {}", k);
    }
}

#[test]
fn e06_49_silence_without_key_on() {
    let mut apu = Apu::new();
    setup(&mut apu, 0, 0x20); // kon=0: no key on
    apu.step_to(1_000_000);
    let v = apu.take_audio();

    for (i, &s) in v.iter().enumerate() {
        assert_eq!(s, 0, "sample {} should be silent: {}", i, s);
    }
}

#[test]
fn e06_49_mute_flag() {
    let mut apu = Apu::new();
    setup(&mut apu, 1, 0x60); // FLG=0x60: mute bit set
    apu.step_to(1_000_000);
    let v = apu.take_audio();

    for (i, &s) in v.iter().enumerate() {
        assert_eq!(s, 0, "sample {} should be silent with mute flag: {}", i, s);
    }
}

#[test]
fn e06_49_endx_set_by_loop() {
    let mut apu = Apu::new();
    setup(&mut apu, 1, 0x20);
    apu.step_to(1_000_000);

    let endx = apu.debug_dsp_read(0x7C);
    assert_eq!(endx & 1, 1, "ENDX bit 0 should be set after loop");
}

#[test]
fn e06_49_start_delay() {
    let mut apu = Apu::new();
    setup(&mut apu, 1, 0x20);
    apu.step_to(1_000_000);
    let v = apu.take_audio();

    // First 6 pairs should be exactly [0,0] (key-on delay)
    for k in 0..6 {
        assert_eq!(v[2 * k], 0, "pair {} left should be 0", k);
        assert_eq!(v[2 * k + 1], 0, "pair {} right should be 0", k);
    }

    // 7th pair (index 6) should be non-zero
    assert!(v[2 * 6] != 0 || v[2 * 6 + 1] != 0,
        "pair 6 should be non-zero after key-on delay");
}

#[test]
fn e06_49_deterministic() {
    let mut apu1 = Apu::new();
    setup(&mut apu1, 1, 0x20);
    apu1.step_to(1_000_000);
    let v1 = apu1.take_audio();

    let mut apu2 = Apu::new();
    setup(&mut apu2, 1, 0x20);
    apu2.step_to(1_000_000);
    let v2 = apu2.take_audio();

    assert_eq!(v1, v2, "audio must be identical across runs");

    // Hashes should also match
    let mut h1 = Fnv64::new();
    apu1.hash_into(&mut h1);
    let mut h2 = Fnv64::new();
    apu2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish(), "state hashes must match");
}

#[test]
fn e06_49_step_to_split_invariant() {
    // Single step_to call
    let mut apu_a = Apu::new();
    setup(&mut apu_a, 1, 0x20);
    apu_a.step_to(1_000_000);
    let v_a = apu_a.take_audio();

    // Split into two step_to calls
    let mut apu_b = Apu::new();
    setup(&mut apu_b, 1, 0x20);
    apu_b.step_to(400_000);
    let v_b1 = apu_b.take_audio();
    apu_b.step_to(1_000_000);
    let v_b2 = apu_b.take_audio();

    // Concatenate split results
    let mut v_b = v_b1;
    v_b.extend(v_b2);

    assert_eq!(v_a, v_b, "split step_to must produce identical audio");
}

#[test]
fn e06_49_smp_program_drives_dsp() {
    let mut apu = Apu::new();

    // Same data setup as before
    apu.debug_poke(0x0200, &[0x00, 0x03, 0x00, 0x03]);
    apu.debug_poke(0x0300, &[0xC3, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77]);

    // Build SMP program that writes DSP registers via $F2/$F3
    let mut prog = Vec::new();
    let pairs = [
        (0x5D, 0x02), (0x04, 0x00), (0x02, 0x00), (0x03, 0x10),
        (0x05, 0x00), (0x07, 0x7F), (0x00, 0x7F), (0x01, 0x7F),
        (0x0C, 0x7F), (0x1C, 0x7F), (0x6C, 0x20),
    ];
    for &(r, v) in &pairs {
        prog.push(0x8F); prog.push(r); prog.push(0xF2); // MOV $F2,#reg
        prog.push(0x8F); prog.push(v); prog.push(0xF3); // MOV $F3,#val
    }
    // KON=0x01 last to start playback
    prog.push(0x8F); prog.push(0x4C); prog.push(0xF2);
    prog.push(0x8F); prog.push(0x01); prog.push(0xF3);
    // Infinite loop: BRA -2
    prog.push(0x2F); prog.push(0xFE);

    apu.debug_poke(0x0400, &prog);
    apu.debug_set_pc(0x0400);

    apu.step_to(1_000_000);
    let v = apu.take_audio();

    // Audio should be non-zero and in expected range at the end
    assert!(!v.is_empty(), "audio should not be empty");
    let last_pair_idx = v.len() - 2;
    assert!(v[last_pair_idx] >= 27000 && v[last_pair_idx] <= 29000,
        "last sample out of range: {}", v[last_pair_idx]);

    // FLG should be set to 0x20 by the program
    let flg = apu.debug_dsp_read(0x6C);
    assert_eq!(flg, 0x20, "FLG should be 0x20");
}
