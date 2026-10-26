use puce16_core::region::Region;
use puce16_core::ppu::{Ppu, DotEvents};
use puce16_core::hash::Fnv64;

fn at(p: &mut Ppu, v: u32, dot: u32) {
    for _ in 0..(v * 340 + dot) {
        p.step_dot();
    }
}

#[test]
fn e04_09_mpy_bytes() {
    let mut p = Ppu::new(Region::Ntsc);
    // a = 256 (write low byte 0x00, then high byte 0x01)
    p.write(0x1B, 0x00);
    p.write(0x1B, 0x01);
    // b_last_byte = 0xFF (-1 as signed byte)
    p.write(0x1C, 0xFF);
    // product = 256 * -1 = -256 = 0xFFFF00 (24-bit two's complement)
    let l = p.read(0x34, 0, false);
    let m = p.read(0x35, 0, false);
    let h = p.read(0x36, 0, false);
    assert_eq!(l, 0x00);
    assert_eq!(m, 0xFF);
    assert_eq!(h, 0xFF);
    assert_eq!(p.ppu1_mdr(), 0xFF);
}

#[test]
fn e04_09_mpy_negative_small() {
    let mut p = Ppu::new(Region::Ntsc);
    // a = -1 (write low byte 0xFF, then high byte 0xFF)
    p.write(0x1B, 0xFF);
    p.write(0x1B, 0xFF);
    // b_last_byte = 0x7F (127 as signed byte)
    p.write(0x1C, 0x7F);
    // product = -1 * 127 = -127 = 0xFFFF81 (24-bit two's complement)
    let l = p.read(0x34, 0, false);
    let m = p.read(0x35, 0, false);
    let h = p.read(0x36, 0, false);
    assert_eq!(l, 0x81);
    assert_eq!(m, 0xFF);
    assert_eq!(h, 0xFF);
}

#[test]
fn e04_09_mpy_positive() {
    let mut p = Ppu::new(Region::Ntsc);
    // a = 256 (write low byte 0x00, then high byte 0x01)
    p.write(0x1B, 0x00);
    p.write(0x1B, 0x01);
    // b_last_byte = 0x7F (127 as signed byte)
    p.write(0x1C, 0x7F);
    // product = 256 * 127 = 32512 = 0x007F00 (24-bit)
    let l = p.read(0x34, 0, false);
    let m = p.read(0x35, 0, false);
    let h = p.read(0x36, 0, false);
    assert_eq!(l, 0x00);
    assert_eq!(m, 0x7F);
    assert_eq!(h, 0x00);
}

#[test]
fn e04_09_oam_read() {
    let mut p = Ppu::new(Region::Ntsc);
    // forced blank is true by default, so OAM reads are allowed
    // write four bytes to OAM at address 0
    p.write(0x02, 0x00);
    p.write(0x03, 0x00);
    p.write(0x04, 0x11);
    p.write(0x04, 0x22);
    p.write(0x04, 0x33);
    p.write(0x04, 0x44);

    // re-address to 0 and read back
    p.write(0x02, 0x00);
    let r1 = p.read(0x38, 0, false);
    let r2 = p.read(0x38, 0, false);
    let r3 = p.read(0x38, 0, false);
    let r4 = p.read(0x38, 0, false);
    assert_eq!(r1, 0x11);
    assert_eq!(r2, 0x22);
    assert_eq!(r3, 0x33);
    assert_eq!(r4, 0x44);
    assert_eq!(p.ppu1_mdr(), 0x44);

    // display on at v=0: OAM reads are blocked (return 0)
    let mut p2 = Ppu::new(Region::Ntsc);
    p2.write(0x00, 0x0F);
    p2.write(0x02, 0x00);
    let r5 = p2.read(0x38, 0x99, false);
    assert_eq!(r5, 0);
}

#[test]
fn e04_09_vram_read_sequence() {
    let mut p = Ppu::new(Region::Ntsc);
    // forced blank is true by default, so VRAM reads are allowed
    // write two words to VRAM at address 0x10 (word index 16)
    p.write(0x15, 0x80);
    p.write(0x16, 0x10);
    p.write(0x17, 0x00);
    p.write(0x18, 0xCD);
    p.write(0x19, 0xAB); // word at 0x10 = 0xABCD
    p.write(0x18, 0x34);
    p.write(0x19, 0x12); // word at 0x11 = 0x1234

    // re-address to 0x10 (triggers prefetch)
    p.write(0x16, 0x10);

    // read low byte of first word
    let r1 = p.read(0x39, 0, false);
    assert_eq!(r1, 0xCD);
    // vmadd should not have changed after reading low byte (inc_on_high is true)
    assert_eq!(p.vram_unit().vmadd(), 0x10);

    // read high byte of first word
    let r2 = p.read(0x3A, 0, false);
    assert_eq!(r2, 0xAB);

    // reading high byte again returns same value (no increment)
    let r3 = p.read(0x3A, 0, false);
    assert_eq!(r3, 0xAB);

    // read low byte of second word (vmadd incremented after high byte read)
    let r4 = p.read(0x39, 0, false);
    assert_eq!(r4, 0x34);

    // read high byte of second word
    let r5 = p.read(0x3A, 0, false);
    assert_eq!(r5, 0x12);
    assert_eq!(p.ppu1_mdr(), 0x12);
}

#[test]
fn e04_09_vram_read_blocked() {
    let mut p = Ppu::new(Region::Ntsc);
    // display on at v=0: VRAM reads are blocked (return 0)
    p.write(0x00, 0x0F);
    p.write(0x16, 0x00);
    p.write(0x17, 0x00);

    let r = p.read(0x39, 0x77, false);
    assert_eq!(r, 0);
    // vmadd should have incremented even though read was blocked (inc_on_high is false)
    assert_eq!(p.vram_unit().vmadd(), 1);
}

#[test]
fn e04_09_cgram_read_bit7_open_bus() {
    let mut p = Ppu::new(Region::Ntsc);
    // write color to CGRAM slot 5: low byte 0xBC, high byte 0x7A
    p.write(0x21, 5);
    p.write(0x22, 0xBC);
    p.write(0x22, 0x7A);

    // re-address to slot 5 and read
    p.write(0x21, 5);
    let r1 = p.read(0x3B, 0, false);
    assert_eq!(r1, 0xBC);
    assert_eq!(p.ppu2_mdr(), 0xBC);

    // second read: bit 7 comes from previous ppu2_mdr (open bus behavior)
    let r2 = p.read(0x3B, 0, false);
    // actual value is 0x7A, but bit 7 is replaced by bit 7 of ppu2_mdr (which was 0xBC -> bit 7 = 1)
    assert_eq!(r2, 0xFA);

    // write color to slot 6: both bytes have bit 7 = 0
    p.write(0x21, 6);
    p.write(0x22, 0x01);
    p.write(0x22, 0x02);
    p.write(0x21, 6);
    let r3 = p.read(0x3B, 0, false);
    assert_eq!(r3, 0x01);
    // now ppu2_mdr has bit 7 = 0, so next read's bit 7 will be 0
    let r4 = p.read(0x3B, 0, false);
    assert_eq!(r4, 0x02);
}

#[test]
fn e04_09_mdr_independent() {
    let mut p = Ppu::new(Region::Ntsc);

    // Set up MPY: a=1, b_last_byte=0xAB (-85) -> product = -85 = 0xFFAB (24-bit)
    p.write(0x1B, 0x01);
    p.write(0x1B, 0x00);
    p.write(0x1C, 0xAB);

    // Read MPY low byte -> sets ppu1_mdr to 0xAB
    let mpy_low = p.read(0x34, 0, false);
    assert_eq!(mpy_low, 0xAB);
    assert_eq!(p.ppu1_mdr(), 0xAB);

    // Write to CGRAM and read -> sets ppu2_mdr but not ppu1_mdr
    p.write(0x21, 0);
    p.write(0x22, 0xCD);
    p.write(0x22, 0xEF);
    p.write(0x21, 0);
    let cgram_val = p.read(0x3B, 0, false);
    assert_eq!(cgram_val, 0xCD);
    // ppu1_mdr should still be from the MPY read (unchanged by CGRAM read)
    assert_eq!(p.ppu1_mdr(), 0xAB);
    assert_eq!(p.ppu2_mdr(), cgram_val);

    // Read MPY again -> sets ppu1_mdr but not ppu2_mdr
    let mpy_low2 = p.read(0x34, 0, false);
    assert_eq!(mpy_low2, 0xAB);
    assert_eq!(p.ppu1_mdr(), 0xAB);
    // ppu2_mdr should still be from the CGRAM read (unchanged by MPY read)
    assert_eq!(p.ppu2_mdr(), cgram_val);
}

#[test]
fn e04_09_open_bus_cpu() {
    let mut p = Ppu::new(Region::Ntsc);
    let base_hash = hash(&p);

    // reads of non-PPU registers return cpu_mdr (open bus)
    assert_eq!(p.read(0x05, 0xAB, false), 0xAB);
    assert_eq!(p.read(0x00, 0x12, true), 0x12);
    assert_eq!(p.read(0x37, 0xCD, false), 0xCD);
    assert_eq!(p.read(0x40, 0x55, false), 0x55);

    // these reads should not change the hash (no state modified)
    assert_eq!(hash(&p), base_hash);
}

#[test]
fn e04_09_hash() {
    let mut p = Ppu::new(Region::Ntsc);
    let base = hash(&p);

    // read MPY with non-zero product -> changes ppu1_mdr from 0
    p.write(0x1B, 0x00);
    p.write(0x1B, 0x01);
    p.write(0x1C, 0x7F);
    let _ = p.read(0x34, 0, false);
    assert_ne!(hash(&p), base);

    // read CGRAM -> changes ppu2_mdr and addr
    let mut p2 = Ppu::new(Region::Ntsc);
    p2.write(0x21, 0);
    p2.write(0x22, 0xAB);
    p2.write(0x22, 0xCD);
    p2.write(0x21, 0);
    let _ = p2.read(0x3B, 0, false);
    assert_ne!(hash(&p2), base);
}

fn hash(p: &Ppu) -> u64 {
    let mut h = Fnv64::new();
    p.hash_into(&mut h);
    h.finish()
}
