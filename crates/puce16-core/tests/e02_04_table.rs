// E02-04: Verify generated cpu/table.rs has correct Mn/Am enums and lookup tables.

use puce16_core::cpu::table::{Mn, Am, MN, AM};

#[test]
fn e02_04_known_ops() {
    // 0x00 -> (Brk, Brk)
    assert_eq!(MN[0x00], Mn::BRK);
    assert_eq!(AM[0x00], Am::Brk);

    // 0x01 -> (Ora, DpXInd)
    assert_eq!(MN[0x01], Mn::ORA);
    assert_eq!(AM[0x01], Am::DpXInd);

    // 0x20 -> (Jsr, Abs)
    assert_eq!(MN[0x20], Mn::JSR);
    assert_eq!(AM[0x20], Am::Abs);

    // 0x22 -> (Jsl, Long)
    assert_eq!(MN[0x22], Mn::JSL);
    assert_eq!(AM[0x22], Am::Long);

    // 0x4C -> (Jmp, Abs)
    assert_eq!(MN[0x4C], Mn::JMP);
    assert_eq!(AM[0x4C], Am::Abs);

    // 0x5C -> (Jml, Long)
    assert_eq!(MN[0x5C], Mn::JML);
    assert_eq!(AM[0x5C], Am::Long);

    // 0x69 -> (Adc, Imm)
    assert_eq!(MN[0x69], Mn::ADC);
    assert_eq!(AM[0x69], Am::Imm);

    // 0x6C -> (Jmp, AbsInd)
    assert_eq!(MN[0x6C], Mn::JMP);
    assert_eq!(AM[0x6C], Am::AbsInd);

    // 0x7C -> (Jmp, AbsXInd)
    assert_eq!(MN[0x7C], Mn::JMP);
    assert_eq!(AM[0x7C], Am::AbsXInd);

    // 0xDC -> (Jml, AbsIndLong)
    assert_eq!(MN[0xDC], Mn::JML);
    assert_eq!(AM[0xDC], Am::AbsIndLong);

    // 0xFC -> (Jsr, AbsXInd)
    assert_eq!(MN[0xFC], Mn::JSR);
    assert_eq!(AM[0xFC], Am::AbsXInd);
}

#[test]
fn e02_04_more_ops() {
    // 0x0A -> (Asl, Acc)
    assert_eq!(MN[0x0A], Mn::ASL);
    assert_eq!(AM[0x0A], Am::Acc);

    // 0x44 -> (Mvp, Blk)
    assert_eq!(MN[0x44], Mn::MVP);
    assert_eq!(AM[0x44], Am::Blk);

    // 0x54 -> (Mvn, Blk)
    assert_eq!(MN[0x54], Mn::MVN);
    assert_eq!(AM[0x54], Am::Blk);

    // 0x62 -> (Per, Per)
    assert_eq!(MN[0x62], Mn::PER);
    assert_eq!(AM[0x62], Am::Per);

    // 0x80 -> (Bra, Rel)
    assert_eq!(MN[0x80], Mn::BRA);
    assert_eq!(AM[0x80], Am::Rel);

    // 0x82 -> (Brl, RelL)
    assert_eq!(MN[0x82], Mn::BRL);
    assert_eq!(AM[0x82], Am::RelL);

    // 0x83 -> (Sta, Sr)
    assert_eq!(MN[0x83], Mn::STA);
    assert_eq!(AM[0x83], Am::Sr);

    // 0x93 -> (Sta, SrIndY)
    assert_eq!(MN[0x93], Mn::STA);
    assert_eq!(AM[0x93], Am::SrIndY);

    // 0xB1 -> (Lda, DpIndY)
    assert_eq!(MN[0xB1], Mn::LDA);
    assert_eq!(AM[0xB1], Am::DpIndY);

    // 0xB7 -> (Lda, DpIndLongY)
    assert_eq!(MN[0xB7], Mn::LDA);
    assert_eq!(AM[0xB7], Am::DpIndLongY);

    // 0xC2 -> (Rep, Imm8)
    assert_eq!(MN[0xC2], Mn::REP);
    assert_eq!(AM[0xC2], Am::Imm8);

    // 0xD4 -> (Pei, Pei)
    assert_eq!(MN[0xD4], Mn::PEI);
    assert_eq!(AM[0xD4], Am::Pei);

    // 0xEA -> (Nop, Imp)
    assert_eq!(MN[0xEA], Mn::NOP);
    assert_eq!(AM[0xEA], Am::Imp);

    // 0xF4 -> (Pea, Pea)
    assert_eq!(MN[0xF4], Mn::PEA);
    assert_eq!(AM[0xF4], Am::Pea);

    // 0x42 -> (Wdm, Wdm)
    assert_eq!(MN[0x42], Mn::WDM);
    assert_eq!(AM[0x42], Am::Wdm);

    // 0xFB -> (Xce, Imp)
    assert_eq!(MN[0xFB], Mn::XCE);
    assert_eq!(AM[0xFB], Am::Imp);
}

#[test]
fn e02_04_names() {
    assert_eq!(Mn::ADC.name(), "ADC");
    assert_eq!(Mn::AND.name(), "AND");
    assert_eq!(Mn::XCE.name(), "XCE");
    assert_eq!(Mn::NOP.name(), "NOP");
}

#[test]
fn e02_04_all_variants_used() {
    // Collect unique Mn values from MN table
    let mut mn_seen = Vec::new();
    for &m in MN.iter() {
        if !mn_seen.contains(&m) {
            mn_seen.push(m);
        }
    }
    assert_eq!(mn_seen.len(), 92, "MN should contain exactly 92 distinct Mn variants");

    // Collect unique Am values from AM table
    let mut am_seen = Vec::new();
    for &a in AM.iter() {
        if !am_seen.contains(&a) {
            am_seen.push(a);
        }
    }
    assert_eq!(am_seen.len(), 31, "AM should contain exactly 31 distinct Am variants");
}
