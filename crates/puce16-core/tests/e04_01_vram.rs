use puce16_core::ppu::vram::{remap, Vram};
use puce16_core::hash::Fnv64;

#[test]
fn e04_01_remap_modes() {
    assert_eq!(remap(0, 0x1234), 0x1234);
    assert_eq!(remap(1, 0x1234), 0x12A1);
    assert_eq!(remap(2, 0x1234), 0x13A0);
    assert_eq!(remap(3, 0x1234), 0x11A4);
    assert_eq!(remap(1, 0x0001), 0x0008);
    assert_eq!(remap(1, 0x0020), 0x0001);
    assert_eq!(remap(2, 0x0040), 0x0001);
    assert_eq!(remap(3, 0x0080), 0x0001);
    assert_eq!(remap(2, 0x0001), 0x0008);
    assert_eq!(remap(3, 0x0001), 0x0008);
    assert_eq!(remap(1, 0x00FF), 0x00FF);
    assert_eq!(remap(2, 0x01FF), 0x01FF);
}

#[test]
fn e04_01_power_on() {
    let v = Vram::new();
    assert_eq!(v.vmain(), 0);
    assert_eq!(v.vmadd(), 0);
    assert_eq!(v.word(0), 0);
    assert_eq!(v.word(0x7FFF), 0);
    assert_eq!(v.mem().len(), 0x8000);
}

#[test]
fn e04_01_step_values() {
    let mut v = Vram::new();
    
    v.set_vmain(0x00);
    assert_eq!(v.step(), 1);
    assert!(!v.inc_on_high());
    
    v.set_vmain(0x01);
    assert_eq!(v.step(), 32);
    
    v.set_vmain(0x02);
    assert_eq!(v.step(), 128);
    
    v.set_vmain(0x03);
    assert_eq!(v.step(), 128);
    
    v.set_vmain(0x80);
    assert_eq!(v.step(), 1);
    assert!(v.inc_on_high());
    
    v.set_vmain(0x8D);
    assert_eq!(v.step(), 32);
    assert!(v.inc_on_high());
}

#[test]
fn e04_01_write_low_inc1() {
    let mut v = Vram::new();
    v.set_vmain(0x00);
    v.set_addr_l(0);
    v.set_addr_h(0);
    
    v.write_low(0x11, true);
    v.write_low(0x22, true);
    assert_eq!(v.word(0), 0x0011);
    assert_eq!(v.word(1), 0x0022);
    assert_eq!(v.vmadd(), 2);
    
    // write_high doesn't increment when inc_on_high is false
    v.write_high(0x33, true);
    assert_eq!(v.word(2), 0x3300);
    assert_eq!(v.vmadd(), 2);
}

#[test]
fn e04_01_write_word_inc_high() {
    let mut v = Vram::new();
    v.set_vmain(0x80);
    v.set_addr_l(0);
    v.set_addr_h(1);
    
    // write_low doesn't increment when inc_on_high is true
    v.write_low(0xCD, true);
    assert_eq!(v.vmadd(), 0x100);
    
    // write_high increments
    v.write_high(0xAB, true);
    assert_eq!(v.word(0x100), 0xABCD);
    assert_eq!(v.vmadd(), 0x101);
    
    // Next word
    v.write_low(0x01, true);
    v.write_high(0x02, true);
    assert_eq!(v.word(0x101), 0x0201);
    assert_eq!(v.vmadd(), 0x102);
}

#[test]
fn e04_01_step32_step128() {
    let mut v = Vram::new();
    
    // Step 32 (VMAIN bit 1 set)
    v.set_vmain(0x81);
    v.set_addr_l(0);
    v.set_addr_h(0);
    
    v.write_low(1, true);
    v.write_high(2, true);
    assert_eq!(v.word(0), 0x0201);
    assert_eq!(v.vmadd(), 32);
    
    v.write_low(3, true);
    v.write_high(4, true);
    assert_eq!(v.word(32), 0x0403);
    assert_eq!(v.vmadd(), 64);
    
    // Step 128 (VMAIN bits 1-0 = 10)
    v.set_vmain(0x82);
    v.set_addr_l(0);
    v.set_addr_h(0);
    
    v.write_low(5, true);
    v.write_high(6, true);
    assert_eq!(v.vmadd(), 128);
    
    v.write_low(7, true);
    v.write_high(8, true);
    assert_eq!(v.vmadd(), 256);
    
    v.write_low(9, true);
    v.write_high(10, true);
    assert_eq!(v.vmadd(), 384);
    assert_ne!(v.word(128), 0);
    assert_ne!(v.word(256), 0);
}

#[test]
fn e04_01_addr_wrap_mask() {
    let mut v = Vram::new();
    v.set_vmain(0x00);
    
    // Address wraps at 0x7FFF
    v.set_addr_l(0xFF);
    v.set_addr_h(0x7F);
    assert_eq!(v.vmadd(), 0x7FFF);
    
    v.write_low(0x5A, true);
    assert_eq!(v.word(0x7FFF), 0x005A);
    assert_eq!(v.vmadd(), 0); // wrapped
    
    // Bit 7 of VMADDH is ignored
    v.set_addr_l(0);
    v.set_addr_h(0xFF);
    assert_eq!(v.vmadd(), 0x7F00);
}

#[test]
fn e04_01_blocked_write() {
    let mut v = Vram::new();
    
    // Blocked write still increments (inc_on_high false)
    v.set_vmain(0x00);
    v.set_addr_l(5);
    v.set_addr_h(0);
    v.write_low(0x77, false);
    assert_eq!(v.word(5), 0);
    assert_eq!(v.vmadd(), 6);
    
    // Blocked write_high increments when inc_on_high true
    v.set_vmain(0x80);
    v.set_addr_l(9);
    v.set_addr_h(0);
    v.write_high(0x88, false);
    assert_eq!(v.word(9), 0);
    assert_eq!(v.vmadd(), 10);
    
    // write_low doesn't increment when inc_on_high true
    v.write_low(0x99, false);
    assert_eq!(v.vmadd(), 10);
}

#[test]
fn e04_01_remap_applied() {
    let mut v = Vram::new();
    
    // Remap mode 1 (VMAIN bit 2 set), step 1, inc on low
    v.set_vmain(0x04);
    v.set_addr_l(1);
    v.set_addr_h(0);
    
    v.write_low(0x99, true);
    // remap(1, 1) = (1 & 0xFF00) | ((1 & 0x00E0) >> 5) | ((1 & 0x001F) << 3) = 8
    assert_eq!(v.word(8), 0x0099);
    assert_eq!(v.word(1), 0); // original address not written to
    assert_eq!(v.vmadd(), 2); // vmadd itself is not remapped
}

#[test]
fn e04_01_prefetch_on_addr() {
    let mut v = Vram::new();
    
    // Prefetch loads latch from VRAM at remapped address
    v.set_word(0x0100, 0xCAFE);
    v.set_vmain(0);
    v.set_addr_l(0);
    v.set_addr_h(1);
    assert_eq!(v.read_low(true), 0xFE); // low byte of latch
    
    // Prefetch uses remap
    v.set_word(8, 0xABCD);
    v.set_vmain(0x04); // remap mode 1
    v.set_addr_l(1);
    v.set_addr_h(0);
    assert_eq!(v.read_low(true), 0xCD); // low byte of word at remapped address
}

#[test]
fn e04_01_read_low_sequence() {
    let mut v = Vram::new();
    
    v.set_word(5, 0xBEEF);
    v.set_word(6, 0x1234);
    v.set_word(7, 0x5678);
    
    v.set_vmain(0);
    v.set_addr_l(5);
    v.set_addr_h(0);
    
    // First read: returns latch (prefetched), then reloads and increments
    assert_eq!(v.read_low(true), 0xEF); // low byte of word at addr 5
    
    // Second read: same word again (latch was reloaded)
    assert_eq!(v.read_low(true), 0xEF);
    
    // Third read: next word
    assert_eq!(v.read_low(true), 0x34);
    
    // Fourth read: next word
    assert_eq!(v.read_low(true), 0x78);
    
    assert_eq!(v.vmadd(), 9);
}

#[test]
fn e04_01_read_high_sequence() {
    let mut v = Vram::new();
    
    v.set_word(10, 0xA1B2);
    v.set_word(11, 0xC3D4);
    
    v.set_vmain(0x80); // inc on high
    v.set_addr_l(10);
    v.set_addr_h(0);
    
    assert_eq!(v.read_high(true), 0xA1); // high byte of word at addr 10
    assert_eq!(v.vmadd(), 11);
    
    // read_low doesn't change latch or vmadd when inc_on_high is true
    assert_eq!(v.read_low(true), 0xB2);
    assert_eq!(v.vmadd(), 11);
    
    // Next high byte read reloads and increments
    assert_eq!(v.read_high(true), 0xA1);
    assert_eq!(v.vmadd(), 12);
    
    assert_eq!(v.read_high(true), 0xC3);
    assert_eq!(v.vmadd(), 13);
}

#[test]
fn e04_01_read_blocked() {
    let mut v = Vram::new();
    
    v.set_word(20, 0x1111);
    v.set_vmain(0);
    v.set_addr_l(20);
    v.set_addr_h(0);
    
    // Blocked read returns 0 but still reloads latch and increments
    assert_eq!(v.read_low(false), 0);
    assert_eq!(v.vmadd(), 21);
    
    // Next unblocked read gets the reloaded value
    assert_eq!(v.read_low(true), 0x11);
}

#[test]
fn e04_01_hash() {
    let mut v1 = Vram::new();
    let v2 = Vram::new();
    
    // Identical VRAMs hash the same
    let mut h1 = Fnv64::new();
    let mut h2 = Fnv64::new();
    v1.hash_into(&mut h1);
    v2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
    
    // Different memory content hashes differently
    v1.set_word(3, 1);
    let mut h3 = Fnv64::new();
    v1.hash_into(&mut h3);
    assert_ne!(h3.finish(), h1.finish());
    
    // Different vmain hashes differently
    let mut v3 = Vram::new();
    v3.set_vmain(0x80);
    let mut h4 = Fnv64::new();
    v3.hash_into(&mut h4);
    assert_ne!(h4.finish(), h1.finish());
    
    // Different vmadd hashes differently
    let mut v4 = Vram::new();
    v4.set_addr_l(1);
    let mut h5 = Fnv64::new();
    v4.hash_into(&mut h5);
    assert_ne!(h5.finish(), h1.finish());
    
    // reset_regs preserves memory
    v1.reset_regs();
    assert_eq!(v1.word(3), 1);
}
