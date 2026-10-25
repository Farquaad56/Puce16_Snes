use puce16_core::dma::{a_step, is_wram_addr, pattern_offsets, step_a, DmaBus};

#[test]
fn e03_18_offsets() {
    assert_eq!(pattern_offsets(0), &[0]);
    assert_eq!(pattern_offsets(1), &[0, 1]);
    assert_eq!(pattern_offsets(2), &[0, 0]);
    assert_eq!(pattern_offsets(3), &[0, 0, 1, 1]);
    assert_eq!(pattern_offsets(4), &[0, 1, 2, 3]);
    assert_eq!(pattern_offsets(5), &[0, 1, 0, 1]);
    assert_eq!(pattern_offsets(6), &[0, 0]);
    assert_eq!(pattern_offsets(7), &[0, 0, 1, 1]);
    // Mask to lower 3 bits: 0x0B & 7 = 3
    assert_eq!(pattern_offsets(0x0B), pattern_offsets(3));
}

#[test]
fn e03_18_a_step() {
    assert_eq!(a_step(0x00), 1);
    assert_eq!(a_step(0x08), 0);
    assert_eq!(a_step(0x10), -1);
    assert_eq!(a_step(0x18), 0);
    // Bits outside 4-3 are ignored
    assert_eq!(a_step(0x81), 1);
    assert_eq!(a_step(0x41), 1);
}

#[test]
fn e03_18_wram_addr() {
    // True cases: banks 7E-7F
    assert!(is_wram_addr(0x7E0000));
    assert!(is_wram_addr(0x7FFFFF));
    // True cases: low bank with address < 0x2000
    assert!(is_wram_addr(0x000000));
    assert!(is_wram_addr(0x001FFF));
    assert!(is_wram_addr(0x3F1234));
    // Bank 80 with low address (bank & 0x7F = 0)
    assert!(is_wram_addr(0x800100));
    // Bank BF with low address (BF & 0x7F = 3F <= 3F)
    assert!(is_wram_addr(0xBF1FFF));

    // False cases: address >= 0x2000 in low bank
    assert!(!is_wram_addr(0x002000));
    assert!(!is_wram_addr(0x00FFFF));
    // Bank 7D is not WRAM
    assert!(!is_wram_addr(0x7D0000));
    // Bank C0: (C0 & 0x7F) = 40 > 3F, and not in 7E-7F range
    assert!(!is_wram_addr(0xC00000));
    // Bank 40: (40 & 0x7F) = 64 > 3F
    assert!(!is_wram_addr(0x400000));
    // Bank 01 with address >= 0x2000
    assert!(!is_wram_addr(0x018000));
}

#[test]
fn e03_18_step_a() {
    assert_eq!(step_a(0x1000, 1), 0x1001);
    // Wrap around at 16 bits
    assert_eq!(step_a(0xFFFF, 1), 0);
    // Underflow wraps to max
    assert_eq!(step_a(0, -1), 0xFFFF);
    // Step of 0 keeps same address
    assert_eq!(step_a(0x1234, 0), 0x1234);
}

#[test]
fn e03_18_trait_object_safe_enough() {
    struct Dummy;
    impl puce16_core::dma::DmaBus for Dummy {
        fn a_read(&mut self, _addr: u32) -> u8 { 0 }
        fn a_write(&mut self, _addr: u32, _v: u8) {}
        fn b_read(&mut self, _reg: u8) -> u8 { 0 }
        fn b_write(&mut self, _reg: u8, _v: u8) {}
        fn elapse(&mut self, _clocks: u32) {}
        fn master(&self) -> u64 { 0 }
    }
    let d = Dummy;
    assert_eq!(d.master(), 0);
}
