// E07-11b: bus cycle positions of every SPC700 opcode, checked against the SingleStepTests oracle patterns.
// blargg spc_mem_access_times measures WHEN inside an instruction each access happens, so the position of
// the internal (wait) cycles matters, not only the set of reads and writes.
use puce16_core::apu::spc700::testbus::{CycleKind, FlatSmpBus};
use puce16_core::apu::spc700::Spc700;
use std::collections::BTreeSet;
// GENERATED from tests/oracle/spc700 (SingleStepTests): bus cycle kinds per opcode.
// R = read, W = write, . = internal (wait) cycle. Several patterns = taken / not taken branch.
const PATTERNS: [&[&str]; 256] = [
    &["RR"], // 00
    &["RR.WW.RR"], // 01
    &["RRRW"], // 02
    &["RRR.R", "RRR.R.."], // 03
    &["RRR"], // 04
    &["RRRR"], // 05
    &["RRR"], // 06
    &["RR.RRR"], // 07
    &["RR"], // 08
    &["RRRRRW"], // 09
    &["RRRR."], // 0A
    &["RRRW"], // 0B
    &["RRRRW"], // 0C
    &["RRW."], // 0D
    &["RRRRRW"], // 0E
    &["RRWWW.RR"], // 0F
    &["RR", "RR.."], // 10
    &["RR.WW.RR"], // 11
    &["RRRW"], // 12
    &["RRR.R", "RRR.R.."], // 13
    &["RR.R"], // 14
    &["RRR.R"], // 15
    &["RRR.R"], // 16
    &["RR.RRR"], // 17
    &["RRRRW"], // 18
    &["RRRRW"], // 19
    &["RRRWRW"], // 1A
    &["RR.RW"], // 1B
    &["RR"], // 1C
    &["RR"], // 1D
    &["RRRR"], // 1E
    &["RRR.RR"], // 1F
    &["RR"], // 20
    &["RR.WW.RR"], // 21
    &["RRRW"], // 22
    &["RRR.R", "RRR.R.."], // 23
    &["RRR"], // 24
    &["RRRR"], // 25
    &["RRR"], // 26
    &["RR.RRR"], // 27
    &["RR"], // 28
    &["RRRRRW"], // 29
    &["RRRR."], // 2A
    &["RRRW"], // 2B
    &["RRRRW"], // 2C
    &["RRW."], // 2D
    &["RRR.R", "RRR.R.."], // 2E
    &["RR.."], // 2F
    &["RR", "RR.."], // 30
    &["RR.WW.RR"], // 31
    &["RRRW"], // 32
    &["RRR.R", "RRR.R.."], // 33
    &["RR.R"], // 34
    &["RRR.R"], // 35
    &["RRR.R"], // 36
    &["RR.RRR"], // 37
    &["RRRRW"], // 38
    &["RRRRW"], // 39
    &["RRRWRW"], // 3A
    &["RR.RW"], // 3B
    &["RR"], // 3C
    &["RR"], // 3D
    &["RRR"], // 3E
    &["RRR.WW.."], // 3F
    &["RR"], // 40
    &["RR.WW.RR"], // 41
    &["RRRW"], // 42
    &["RRR.R", "RRR.R.."], // 43
    &["RRR"], // 44
    &["RRRR"], // 45
    &["RRR"], // 46
    &["RR.RRR"], // 47
    &["RR"], // 48
    &["RRRRRW"], // 49
    &["RRRR"], // 4A
    &["RRRW"], // 4B
    &["RRRRW"], // 4C
    &["RRW."], // 4D
    &["RRRRRW"], // 4E
    &["RR.WW."], // 4F
    &["RR", "RR.."], // 50
    &["RR.WW.RR"], // 51
    &["RRRW"], // 52
    &["RRR.R", "RRR.R.."], // 53
    &["RR.R"], // 54
    &["RRR.R"], // 55
    &["RRR.R"], // 56
    &["RR.RRR"], // 57
    &["RRRRW"], // 58
    &["RRRRW"], // 59
    &["RRRR"], // 5A
    &["RR.RW"], // 5B
    &["RR"], // 5C
    &["RR"], // 5D
    &["RRRR"], // 5E
    &["RRR"], // 5F
    &["RR"], // 60
    &["RR.WW.RR"], // 61
    &["RRRW"], // 62
    &["RRR.R", "RRR.R.."], // 63
    &["RRR"], // 64
    &["RRRR"], // 65
    &["RRR"], // 66
    &["RR.RRR"], // 67
    &["RR"], // 68
    &["RRRRR."], // 69
    &["RRRR"], // 6A
    &["RRRW"], // 6B
    &["RRRRW"], // 6C
    &["RRW."], // 6D
    &["RRRWR", "RRRWR.."], // 6E
    &["RR.RR"], // 6F
    &["RR", "RR.."], // 70
    &["RR.WW.RR"], // 71
    &["RRRW"], // 72
    &["RRR.R", "RRR.R.."], // 73
    &["RR.R"], // 74
    &["RRR.R"], // 75
    &["RRR.R"], // 76
    &["RR.RRR"], // 77
    &["RRRR."], // 78
    &["RRRR."], // 79
    &["RRR.R"], // 7A
    &["RR.RW"], // 7B
    &["RR"], // 7C
    &["RR"], // 7D
    &["RRR"], // 7E
    &["RR.RRR"], // 7F
    &["RR"], // 80
    &["RR.WW.RR"], // 81
    &["RRRW"], // 82
    &["RRR.R", "RRR.R.."], // 83
    &["RRR"], // 84
    &["RRRR"], // 85
    &["RRR"], // 86
    &["RR.RRR"], // 87
    &["RR"], // 88
    &["RRRRRW"], // 89
    &["RRRR."], // 8A
    &["RRRW"], // 8B
    &["RRRRW"], // 8C
    &["RR"], // 8D
    &["RR.R"], // 8E
    &["RRRRW"], // 8F
    &["RR", "RR.."], // 90
    &["RR.WW.RR"], // 91
    &["RRRW"], // 92
    &["RRR.R", "RRR.R.."], // 93
    &["RR.R"], // 94
    &["RRR.R"], // 95
    &["RRR.R"], // 96
    &["RR.RRR"], // 97
    &["RRRRW"], // 98
    &["RRRRW"], // 99
    &["RRR.R"], // 9A
    &["RR.RW"], // 9B
    &["RR"], // 9C
    &["RR"], // 9D
    &["RR.........."], // 9E
    &["RR..."], // 9F
    &["RR."], // A0
    &["RR.WW.RR"], // A1
    &["RRRW"], // A2
    &["RRR.R", "RRR.R.."], // A3
    &["RRR"], // A4
    &["RRRR"], // A5
    &["RRR"], // A6
    &["RR.RRR"], // A7
    &["RR"], // A8
    &["RRRRRW"], // A9
    &["RRRR"], // AA
    &["RRRW"], // AB
    &["RRRRW"], // AC
    &["RR"], // AD
    &["RR.R"], // AE
    &["RR.W"], // AF
    &["RR", "RR.."], // B0
    &["RR.WW.RR"], // B1
    &["RRRW"], // B2
    &["RRR.R", "RRR.R.."], // B3
    &["RR.R"], // B4
    &["RRR.R"], // B5
    &["RRR.R"], // B6
    &["RR.RRR"], // B7
    &["RRRRW"], // B8
    &["RRRRW"], // B9
    &["RRR.R"], // BA
    &["RR.RW"], // BB
    &["RR"], // BC
    &["RR"], // BD
    &["RR."], // BE
    &["RRR."], // BF
    &["RR."], // C0
    &["RR.WW.RR"], // C1
    &["RRRW"], // C2
    &["RRR.R", "RRR.R.."], // C3
    &["RRRW"], // C4
    &["RRRRW"], // C5
    &["RRRW"], // C6
    &["RR.RRRW"], // C7
    &["RR"], // C8
    &["RRRRW"], // C9
    &["RRRR.W"], // CA
    &["RRRW"], // CB
    &["RRRRW"], // CC
    &["RR"], // CD
    &["RR.R"], // CE
    &["RR......."], // CF
    &["RR", "RR.."], // D0
    &["RR.WW.RR"], // D1
    &["RRRW"], // D2
    &["RRR.R", "RRR.R.."], // D3
    &["RR.RW"], // D4
    &["RRR.RW"], // D5
    &["RRR.RW"], // D6
    &["RRRR.RW"], // D7
    &["RRRW"], // D8
    &["RR.RW"], // D9
    &["RRRWW"], // DA
    &["RR.RW"], // DB
    &["RR"], // DC
    &["RR"], // DD
    &["RR.R.R", "RR.R.R.."], // DE
    &["RR."], // DF
    &["RR"], // E0
    &["RR.WW.RR"], // E1
    &["RRRW"], // E2
    &["RRR.R", "RRR.R.."], // E3
    &["RRR"], // E4
    &["RRRR"], // E5
    &["RRR"], // E6
    &["RR.RRR"], // E7
    &["RR"], // E8
    &["RRRR"], // E9
    &["RRRRW"], // EA
    &["RRR"], // EB
    &["RRRR"], // EC
    &["RR."], // ED
    &["RR.R"], // EE
    &["RR.R.R."], // EF
    &["RR", "RR.."], // F0
    &["RR.WW.RR"], // F1
    &["RRRW"], // F2
    &["RRR.R", "RRR.R.."], // F3
    &["RR.R"], // F4
    &["RRR.R"], // F5
    &["RRR.R"], // F6
    &["RR.RRR"], // F7
    &["RRR"], // F8
    &["RR.R"], // F9
    &["RRRRW"], // FA
    &["RR.R"], // FB
    &["RR"], // FC
    &["RR"], // FD
    &["RR.R", "RR.R.."], // FE
    &["RR.R.R."], // FF
];

fn kinds(bus: &FlatSmpBus) -> String {
    bus.log
        .iter()
        .map(|c| match c.kind {
            CycleKind::Read => 'R',
            CycleKind::Write => 'W',
            CycleKind::Idle => '.',
        })
        .collect()
}

/// Cycle patterns produced by the emulator for one opcode over several register/memory setups
/// (so that both taken and not taken branches are seen).
fn emulator_patterns(op: u8) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for fill in [0x00u8, 0x01, 0xFF] {
        for psw in [0x00u8, 0xFF] {
            for (a, x, y) in [(0u8, 0u8, 0u8), (0xFF, 0xFF, 0xFF), (1, 1, 1)] {
                let mut bus = FlatSmpBus::new();
                bus.poke(0x0000, &[fill; 0x40]);
                bus.poke(0x01E0, &[fill; 0x20]);
                bus.poke(0x2000, &[fill; 0x40]);
                bus.poke(0x0200, &[op, 0x10, 0x20]);
                let mut cpu = Spc700::new();
                cpu.pc = 0x0200;
                cpu.sp = 0xEF;
                cpu.psw = psw;
                cpu.a = a;
                cpu.x = x;
                cpu.y = y;
                bus.clear_log();
                cpu.step(&mut bus);
                set.insert(kinds(&bus));
            }
        }
    }
    set
}

#[test]
fn e07_11b_access_cycles_regression() {
    let mut bad = Vec::new();
    for op in 0..=255u8 {
        // SLEEP / STOP halt the core: one dummy cycle, not comparable.
        if op == 0xEF || op == 0xFF {
            continue;
        }
        let want: BTreeSet<String> = PATTERNS[op as usize].iter().map(|s| s.to_string()).collect();
        let got = emulator_patterns(op);
        // every pattern the emulator produces must exist in the oracle, and every oracle pattern must be reachable
        if got != want {
            bad.push(format!("{:02X}: oracle {:?} emulator {:?}", op, want, got));
        }
    }
    assert!(bad.is_empty(), "{} opcodes differ:\n{}", bad.len(), bad.join("\n"));
}