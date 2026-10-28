use puce16_core::apu::io::ApuIo;
use puce16_core::apu::spc700::SmpBus;
use puce16_core::hash::Fnv64;

#[test]
fn e06_34_ipl_visible_default() {
    let mut io = ApuIo::new();
    assert_eq!(io.read(0xFFC0), 0xCD);
    io.write(0xFFC0, 0x11);
    assert_eq!(io.read(0xFFC0), 0xCD);
    io.write(0xF1, 0x00);
    assert_eq!(io.read(0xFFC0), 0x11);
    io.write(0xF1, 0x80);
    assert_eq!(io.read(0xFFC0), 0xCD);
}

#[test]
fn e06_34_ram_plain() {
    let mut io = ApuIo::new();
    io.write(0x0200, 0x5A);
    assert_eq!(io.read(0x0200), 0x5A);
    io.write(0xF8, 0x12);
    assert_eq!(io.read(0xF8), 0x12);
    io.write(0xF9, 0x34);
    assert_eq!(io.read(0xF9), 0x34);
}

#[test]
fn e06_34_test_and_control_read_zero() {
    let mut io = ApuIo::new();
    assert_eq!(io.read(0xF0), 0);
    io.write(0xF1, 0x80);
    assert_eq!(io.read(0xF1), 0);
}

#[test]
fn e06_34_ports_directions() {
    let mut io = ApuIo::new();
    io.port_in[0] = 0x11;
    assert_eq!(io.read(0xF4), 0x11);
    io.write(0xF4, 0x22);
    assert_eq!(io.port_out[0], 0x22);
    assert_eq!(io.port_in[0], 0x11);

    io.port_in[3] = 0x33;
    assert_eq!(io.read(0xF7), 0x33);
    io.write(0xF7, 0x44);
    assert_eq!(io.port_out[3], 0x44);
    assert_eq!(io.port_in[3], 0x33);
}

#[test]
fn e06_34_control_clears_inputs() {
    let mut io = ApuIo::new();
    io.port_in = [1, 2, 3, 4];
    io.write(0xF1, 0x10);
    assert_eq!(io.port_in, [0, 0, 3, 4]);

    io.port_in = [1, 2, 3, 4];
    io.write(0xF1, 0x20);
    assert_eq!(io.port_in, [1, 2, 0, 0]);

    io.port_in = [1, 2, 3, 4];
    io.write(0xF1, 0x30);
    assert_eq!(io.port_in, [0, 0, 0, 0]);
}

#[test]
fn e06_34_dsp_access() {
    let mut io = ApuIo::new();
    io.write(0xF2, 0x0C);
    io.write(0xF3, 0x55);
    assert_eq!(io.read(0xF3), 0x55);
    assert_eq!(io.dsp.read(0x0C), 0x55);

    io.write(0xF2, 0x8C);
    io.write(0xF3, 0x66);
    assert_eq!(io.dsp.read(0x0C), 0x55);
    assert_eq!(io.read(0xF2), 0x0C);
}

#[test]
fn e06_34_timer_targets_read_zero() {
    let mut io = ApuIo::new();
    io.write(0xFA, 3);
    io.write(0xFB, 4);
    io.write(0xFC, 5);
    assert_eq!(io.read(0xFA), 0);
    assert_eq!(io.read(0xFB), 0);
    assert_eq!(io.read(0xFC), 0);
    assert_eq!(io.read(0xFD), 0);
    assert_eq!(io.read(0xFE), 0);
    assert_eq!(io.read(0xFF), 0);
}

#[test]
fn e06_34_timer2_counts() {
    let mut io = ApuIo::new();
    io.write(0xFC, 1);
    io.write(0xF1, 0x04);
    for _ in 0..13 {
        io.idle();
    }
    assert_eq!(io.read(0xFF), 0);

    let mut io2 = ApuIo::new();
    io2.write(0xFC, 1);
    io2.write(0xF1, 0x04);
    for _ in 0..14 {
        io2.idle();
    }
    assert_eq!(io2.read(0xFF), 1);
    assert_eq!(io2.read(0xFF), 0);
}

#[test]
fn e06_34_timer_enable_transition() {
    let mut io = ApuIo::new();
    io.write(0xFC, 1);
    io.write(0xF1, 0x04);
    for _ in 0..14 {
        io.idle();
    }
    assert_eq!(io.timers[2].out(), 1);
    io.write(0xF1, 0x00);
    io.write(0xF1, 0x04);
    assert_eq!(io.read(0xFF), 0);
}

#[test]
fn e06_34_timer0_divider_128() {
    let mut io = ApuIo::new();
    io.write(0xFA, 1);
    io.write(0xF1, 0x01);
    for _ in 0..125 {
        io.idle();
    }
    assert_eq!(io.timers[0].out(), 0);

    let mut io2 = ApuIo::new();
    io2.write(0xFA, 1);
    io2.write(0xF1, 0x01);
    for _ in 0..126 {
        io2.idle();
    }
    assert_eq!(io2.timers[0].out(), 1);
}

#[test]
fn e06_34_reset() {
    let mut io = ApuIo::new();
    io.port_in = [1, 2, 3, 4];
    io.port_out = [5, 6, 7, 8];
    io.control = 0;
    io.write(0x0200, 7);
    io.reset();
    assert_eq!(io.port_in, [0, 0, 0, 0]);
    assert_eq!(io.port_out, [0, 0, 0, 0]);
    assert_eq!(io.control, 0xB0);
    assert_eq!(io.read(0x0200), 7);
}

#[test]
fn e06_34_hash() {
    let io1 = ApuIo::new();
    let mut h1 = Fnv64::default();
    io1.hash_into(&mut h1);
    let hash1 = h1.finish();

    let io2 = ApuIo::new();
    let mut h2 = Fnv64::default();
    io2.hash_into(&mut h2);
    let hash2 = h2.finish();
    assert_eq!(hash1, hash2);

    let mut io3 = ApuIo::new();
    io3.write(0x0200, 1);
    let mut h3 = Fnv64::default();
    io3.hash_into(&mut h3);
    let hash3 = h3.finish();
    assert_ne!(hash1, hash3);
}
