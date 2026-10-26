use puce16_core::ppu::cgram::Cgram;
use puce16_core::hash::Fnv64;

#[test]
fn e04_02_set_addr_resets_toggle() {
    let mut c = Cgram::new();
    // Write one byte to set toggle=true
    c.write(0x12);
    assert!(c.toggle());
    
    // set_addr should reset toggle to false
    c.set_addr(5);
    assert_eq!(c.addr(), 5);
    assert!(!c.toggle());
}

#[test]
fn e04_02_pair_write() {
    let mut c = Cgram::new();
    
    // Write first pair: low=0x1F, high=0x03 -> word should be 0x031F
    c.set_addr(0);
    c.write(0x1F);
    c.write(0x03);
    assert_eq!(c.word(0), 0x031F);
    assert_eq!(c.addr(), 1);
    assert!(!c.toggle());
    
    // Write second pair: low=0xFF, high=0xFF -> word should be 0x7FFF (bit 15 masked)
    c.write(0xFF);
    c.write(0xFF);
    assert_eq!(c.word(1), 0x7FFF);
    assert_eq!(c.addr(), 2);
}

#[test]
fn e04_02_latch_not_committed() {
    let mut c = Cgram::new();
    
    // Write only the low byte - word should not be committed yet
    c.set_addr(3);
    c.write(0xAA);
    assert_eq!(c.word(3), 0);
    assert!(c.toggle());
    assert_eq!(c.addr(), 3);
    
    // Now write the high byte to commit
    c.write(0x01);
    assert_eq!(c.word(3), 0x01AA);
    assert_eq!(c.addr(), 4);
}

#[test]
fn e04_02_addr_wrap() {
    let mut c = Cgram::new();
    
    // Write at address 255, should wrap to 0 after commit
    c.set_addr(255);
    c.write(0x01);
    c.write(0x02);
    assert_eq!(c.word(255), 0x0201);
    assert_eq!(c.addr(), 0);
}

#[test]
fn e04_02_read_pair() {
    let mut c = Cgram::new();
    
    // Set up a word to read: 0x7ABC at index 2
    c.set_word(2, 0x7ABC);
    c.set_addr(2);
    
    // First read (toggle=false): should return low byte 0xBC
    let b1 = c.read(0x00);
    assert_eq!(b1, 0xBC);
    assert!(c.toggle());
    assert_eq!(c.addr(), 2); // addr doesn't increment on low byte read
    
    // Second read (toggle=true): should return high byte with bit 15 masked -> 0x7A
    let b2 = c.read(0x00);
    assert_eq!(b2, 0x7A);
    assert!(!c.toggle());
    assert_eq!(c.addr(), 3); // addr increments after high byte read
    
    // Same sequence with m=0x80 on the second read: bit 15 should be set -> 0xFA
    c.set_word(4, 0x7ABC);
    c.set_addr(4);
    let _ = c.read(0x00); // low byte
    let b3 = c.read(0x80); // high byte with MSB from previous result
    assert_eq!(b3, 0xFA);
}

#[test]
fn e04_02_set_word_masks_bit15() {
    let mut c = Cgram::new();
    
    // set_word should mask bit 15
    c.set_word(0, 0xFFFF);
    assert_eq!(c.word(0), 0x7FFF);
}

#[test]
fn e04_02_shared_toggle() {
    let mut c = Cgram::new();
    
    // Write low byte (toggle becomes true)
    c.set_addr(0);
    c.write(0x11);
    assert!(c.toggle());
    
    // Read while toggle is true: treated as high byte read of word at addr 0
    // word(0) is still 0, so high byte is 0 -> result is 0 | (m & 0x80) = 0
    let b = c.read(0x00);
    assert_eq!(b, 0);
    assert!(!c.toggle());
    assert_eq!(c.addr(), 1); // addr incremented after high byte read
}

#[test]
fn e04_02_hash() {
    let mut c1 = Cgram::new();
    let mut c2 = Cgram::new();
    
    // Identical CGRAMs hash the same
    let mut h1 = Fnv64::new();
    let mut h2 = Fnv64::new();
    c1.hash_into(&mut h1);
    c2.hash_into(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
    
    // Different memory content hashes differently
    c1.set_word(0, 1);
    let mut h3 = Fnv64::new();
    c1.hash_into(&mut h3);
    assert_ne!(h3.finish(), h1.finish());
    
    // Different addr hashes differently
    let mut c3 = Cgram::new();
    c3.set_addr(1);
    let mut h4 = Fnv64::new();
    c3.hash_into(&mut h4);
    assert_ne!(h4.finish(), h1.finish());
    
    // Different latch/toggle hashes differently (write one byte only)
    let mut c4 = Cgram::new();
    c4.write(5);
    let mut h5 = Fnv64::new();
    c4.hash_into(&mut h5);
    assert_ne!(h5.finish(), h1.finish());
}
