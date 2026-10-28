use puce16_core::apu::spc700::table::{BYTES, CYCLES, BRANCH_EXTRA, NAME};

#[test]
fn e06_04_known_rows() {
    assert_eq!(NAME[0x00], "NOP");
    assert_eq!(NAME[0xE4], "MOV A,dp");
    assert_eq!(NAME[0xC7], "MOV [dp+X],A");
    assert_eq!(NAME[0xFF], "STOP");
    assert_eq!(NAME[0x3F], "CALL !abs");
    assert_eq!(NAME[0xFE], "DBNZ Y,rel");
}

#[test]
fn e06_04_cycles_bytes() {
    // Cycles
    assert_eq!(CYCLES[0x00], 2);   // NOP
    assert_eq!(CYCLES[0x01], 8);   // TCALL 0
    assert_eq!(CYCLES[0x0F], 8);   // BRK
    assert_eq!(CYCLES[0x9E], 12);  // DIV YA,X
    assert_eq!(CYCLES[0xCF], 9);   // MUL YA
    assert_eq!(CYCLES[0xE4], 3);   // MOV A,dp
    assert_eq!(CYCLES[0xC7], 7);   // MOV [dp+X],A

    // Bytes
    assert_eq!(BYTES[0x00], 1);    // NOP
    assert_eq!(BYTES[0x03], 3);    // BBS dp.0,rel
    assert_eq!(BYTES[0x05], 3);    // OR A,!abs
    assert_eq!(BYTES[0x08], 2);    // OR A,#imm
    assert_eq!(BYTES[0x3F], 3);    // CALL !abs
}

#[test]
fn e06_04_branch_extra() {
    assert_eq!(BRANCH_EXTRA[0x03], 2);   // BBS dp.0,rel
    assert_eq!(BRANCH_EXTRA[0x10], 2);   // BPL rel
    assert_eq!(BRANCH_EXTRA[0x2E], 2);   // CBNE dp,rel
    assert_eq!(BRANCH_EXTRA[0xFE], 2);   // DBNZ Y,rel
    assert_eq!(BRANCH_EXTRA[0x2F], 0);   // BRA rel (fixed 4 cycles)
    assert_eq!(BRANCH_EXTRA[0x00], 0);   // NOP
}

#[test]
fn e06_04_all_rows_sane() {
    let mut branch_count = 0;
    for i in 0..256 {
        let idx = i as usize;
        assert!((1..=3).contains(&BYTES[idx]), "BYTES[{:02X}] out of range", i);
        assert!((2..=12).contains(&CYCLES[idx]), "CYCLES[{:02X}] out of range", i);
        assert!(!NAME[idx].is_empty(), "NAME[{:02X}] empty", i);

        if BRANCH_EXTRA[idx] == 2 {
            branch_count += 1;
        } else {
            assert_eq!(BRANCH_EXTRA[idx], 0, "BRANCH_EXTRA[{:02X}] not 0 or 2", i);
        }
    }
    // 8 conditional branches + 16 BBS/BBC + 2 CBNE + 2 DBNZ = 28
    assert_eq!(branch_count, 28, "expected 28 branch opcodes with extra cycles");
}
