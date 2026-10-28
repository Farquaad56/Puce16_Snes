use puce16_core::apu::ipl::IPL_ROM;

#[test]
fn e06_33_length_and_sum() {
    assert_eq!(IPL_ROM.len(), 64);
    let sum: u32 = IPL_ROM.iter().map(|&b| b as u32).sum();
    assert_eq!(sum, 10936);
}

#[test]
fn e06_33_known_bytes() {
    // MOV X,#$EF (first instruction)
    assert_eq!(IPL_ROM[0], 0xCD);
    assert_eq!(IPL_ROM[1], 0xEF);
    // MOV $F4,#$AA
    assert_eq!(&IPL_ROM[9..12], &[0x8F, 0xAA, 0xF4]);
    // MOVW YA,$F6 at $FFEF (part of the sequence)
    assert_eq!(IPL_ROM[47], 0xBA);
    // JMP [$0000+X] (final jump to user code)
    assert_eq!(IPL_ROM[59], 0x1F);
}

#[test]
fn e06_33_reset_vector() {
    // Reset vector at $FFFE/$FFFF points to $FFC0
    assert_eq!(IPL_ROM[62], 0xC0);
    assert_eq!(IPL_ROM[63], 0xFF);
}
