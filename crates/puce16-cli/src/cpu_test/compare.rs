use crate::cpu_test::oracle::{Case, CycleRec, CpuState};
use puce16_core::cpu::testbus::{Cycle, CycleKind, FlatBus};
use puce16_core::cpu::Cpu;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Level { Regs, Count, Full }

fn fmt_u8(v: u8) -> String { format!("0x{:02X}", v) }
fn fmt_u16(v: u16) -> String { format!("0x{:04X}", v) }
fn fmt_u32(v: u32) -> String { format!("0x{:06X}", v) }

pub fn diff_state(expected: &CpuState, cpu: &Cpu, bus: &FlatBus) -> Vec<String> {
    let mut lines = Vec::new();
    if expected.a != cpu.a {
        lines.push(format!("a: expected {} got {}", fmt_u16(expected.a), fmt_u16(cpu.a)));
    }
    if expected.x != cpu.x {
        lines.push(format!("x: expected {} got {}", fmt_u16(expected.x), fmt_u16(cpu.x)));
    }
    if expected.y != cpu.y {
        lines.push(format!("y: expected {} got {}", fmt_u16(expected.y), fmt_u16(cpu.y)));
    }
    if expected.s != cpu.s {
        lines.push(format!("s: expected {} got {}", fmt_u16(expected.s), fmt_u16(cpu.s)));
    }
    if expected.d != cpu.d {
        lines.push(format!("d: expected {} got {}", fmt_u16(expected.d), fmt_u16(cpu.d)));
    }
    if expected.dbr != cpu.db {
        lines.push(format!("db(dbr): expected {} got {}", fmt_u8(expected.dbr), fmt_u8(cpu.db)));
    }
    if expected.pbr != cpu.pb {
        lines.push(format!("pb(pbr): expected {} got {}", fmt_u8(expected.pbr), fmt_u8(cpu.pb)));
    }
    if expected.pc != cpu.pc {
        lines.push(format!("pc: expected {} got {}", fmt_u16(expected.pc), fmt_u16(cpu.pc)));
    }
    if expected.p != cpu.p {
        lines.push(format!("p: expected {} got {}", fmt_u8(expected.p), fmt_u8(cpu.p)));
    }
    if expected.e != cpu.e {
        let e_exp = if expected.e { "1" } else { "0" };
        let e_got = if cpu.e { "1" } else { "0" };
        lines.push(format!("e: expected {} got {}", e_exp, e_got));
    }
    for (addr, val) in &expected.ram {
        let got = bus.peek(*addr);
        if *val != got {
            lines.push(format!(
                "ram[{}]: expected {} got {}",
                fmt_u32(*addr), fmt_u8(*val), fmt_u8(got)
            ));
        }
    }
    lines
}

fn cycle_kind_str(kind: CycleKind) -> &'static str {
    match kind {
        CycleKind::Read => "R",
        CycleKind::Write => "W",
        CycleKind::Idle => "I",
    }
}

fn expected_cycle_kind(rec: &CycleRec) -> Option<CycleKind> {
    if rec.val.is_none() {
        return Some(CycleKind::Idle);
    }
    let last = rec.kind.chars().last()?;
    match last {
        'r' => Some(CycleKind::Read),
        'w' => Some(CycleKind::Write),
        _ => None,
    }
}

pub fn diff_cycles(expected: &[CycleRec], got: &[Cycle], level: Level) -> Vec<String> {
    if level == Level::Regs {
        return Vec::new();
    }
    let mut lines = Vec::new();
    // Filter out dead cycles (kind == "--------")
    let expected_filtered: Vec<&CycleRec> = expected.iter()
        .filter(|r| r.kind != "--------")
        .collect();

    if level == Level::Count {
        if expected_filtered.len() != got.len() {
            lines.push(format!(
                "cycles: expected {} got {}",
                expected_filtered.len(), got.len()
            ));
        }
        return lines;
    }

    // Full comparison
    if expected_filtered.len() != got.len() {
        lines.push(format!(
            "cycles: expected {} got {}",
            expected_filtered.len(), got.len()
        ));
    }
    let min_len = std::cmp::min(expected_filtered.len(), got.len());
    for i in 0..min_len {
        if lines.len() >= 5 { break; }
        let exp = expected_filtered[i];
        let g = &got[i];
        let exp_kind = match expected_cycle_kind(exp) {
            Some(k) => k,
            None => continue, // Unknown kind, skip
        };
        if exp_kind != g.kind {
            lines.push(format!(
                "cycle {}: expected {} got {}",
                i, cycle_kind_str(exp_kind), cycle_kind_str(g.kind)
            ));
            continue;
        }
        if exp_kind == CycleKind::Idle {
            continue; // Internal cycles don't compare addr/val
        }
        let exp_addr = exp.addr.unwrap_or(0);
        let exp_val = exp.val.unwrap_or(0);
        if exp_addr != g.addr || exp_val != g.val {
            lines.push(format!(
                "cycle {}: expected {} {}={} got {} {}={}",
                i, cycle_kind_str(exp_kind), fmt_u32(exp_addr), fmt_u8(exp_val),
                cycle_kind_str(g.kind), fmt_u32(g.addr), fmt_u8(g.val)
            ));
        }
    }
    lines.truncate(5);
    lines
}

pub fn apply_initial(st: &CpuState, cpu: &mut Cpu, bus: &mut FlatBus) {
    bus.reset_mem();
    for (addr, val) in &st.ram {
        let mut b = [0u8; 1];
        b[0] = *val;
        bus.poke(*addr, &b);
    }
    cpu.a = st.a;
    cpu.x = st.x;
    cpu.y = st.y;
    cpu.d = st.d;
    cpu.db = st.dbr;
    cpu.pb = st.pbr;
    cpu.pc = st.pc;
    cpu.p = st.p;
    cpu.e = st.e;
    if st.e {
        cpu.s = 0x0100 | (st.s & 0xFF);
    } else {
        cpu.s = st.s;
    }
    cpu.halted = false;
    cpu.waiting = false;
}

pub fn is_block_move(opcode: u8) -> bool {
    opcode == 0x44 || opcode == 0x54
}

pub fn run_case(case: &Case, cpu: &mut Cpu, bus: &mut FlatBus, level: Level) -> Vec<String> {
    apply_initial(&case.initial, cpu, bus);
    bus.clear_log();

    let op = bus.peek(cpu.pc24());
    if is_block_move(op) {
        // Block move: run until PC changes or max iterations
        let initial_pc = case.initial.pc;
        let initial_pb = case.initial.pbr;
        for _ in 0..70000 {
            if cpu.pb != initial_pb || cpu.pc != initial_pc {
                break;
            }
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cpu.step(bus);
            })) {
                Ok(()) => {}
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "unknown panic".to_string()
                    };
                    return vec![format!("panic: {}", msg)];
                }
            }
        }
    } else {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cpu.step(bus);
        })) {
            Ok(()) => {}
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "unknown panic".to_string()
                };
                return vec![format!("panic: {}", msg)];
            }
        }
    }

    let mut result = diff_state(&case.final_state, cpu, bus);
    result.extend(diff_cycles(&case.cycles, &bus.log, level));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu_test::oracle::{CycleRec, CpuState};

    fn make_cpu_state() -> CpuState {
        CpuState {
            pc: 0x1234, s: 0x01FF, p: 0x34, a: 0xABCD, x: 0x5678, y: 0x9ABC,
            dbr: 0x00, d: 0x0000, pbr: 0x00, e: true, ram: Vec::new(),
        }
    }

    fn make_cpu() -> Cpu {
        let mut cpu = Cpu::new();
        cpu.a = 0xABCD;
        cpu.x = 0x5678;
        cpu.y = 0x9ABC;
        cpu.s = 0x01FF;
        cpu.d = 0x0000;
        cpu.db = 0x00;
        cpu.pb = 0x00;
        cpu.pc = 0x1234;
        cpu.p = 0x34;
        cpu.e = true;
        cpu
    }

    #[test]
    fn e02_13_diff_state_equal() {
        let st = make_cpu_state();
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        apply_initial(&st, &mut cpu, &mut bus);
        let diffs = diff_state(&st, &cpu, &bus);
        assert!(diffs.is_empty(), "expected no diffs, got: {:?}", diffs);
    }

    #[test]
    fn e02_13_diff_state_fields() {
        let mut st = make_cpu_state();
        st.a = 0x1234;
        st.pc = 0x5678;
        let cpu = make_cpu(); // a=0xABCD, pc=0x1234
        let bus = FlatBus::new();
        let diffs = diff_state(&st, &cpu, &bus);
        assert_eq!(diffs.len(), 2, "expected 2 diffs, got: {:?}", diffs);
        assert!(diffs[0].starts_with("a: expected 0x1234 got 0xABCD"), "first line wrong: {}", diffs[0]);
    }

    #[test]
    fn e02_13_diff_state_ram() {
        let mut st = make_cpu_state();
        st.ram.push((0x7E1234, 0x41));
        let mut cpu = make_cpu();
        let mut bus = FlatBus::new();
        apply_initial(&st, &mut cpu, &mut bus);
        // Write different value to that address
        let mut b = [0u8; 1];
        b[0] = 0x42;
        bus.poke(0x7E1234, &b);
        let diffs = diff_state(&st, &cpu, &bus);
        assert_eq!(diffs.len(), 1, "expected 1 ram diff, got: {:?}", diffs);
        assert!(diffs[0].contains("ram[0x7E1234]"), "wrong line: {}", diffs[0]);
    }

    #[test]
    fn e02_13_diff_cycles_count() {
        let expected = vec![
            CycleRec { addr: Some(0x8000), val: Some(0xEA), kind: "rrr".to_string() },
            CycleRec { addr: None, val: None, kind: "iii".to_string() },
            CycleRec { addr: None, val: None, kind: "iii".to_string() },
        ];
        let got = vec![
            Cycle { addr: 0x8000, val: 0xEA, kind: CycleKind::Read },
            Cycle { addr: 0, val: 0, kind: CycleKind::Idle },
        ];
        // Count level: expected 3 (after filtering dead), got 2
        let diffs = diff_cycles(&expected, &got, Level::Count);
        assert_eq!(diffs.len(), 1, "expected 1 count diff");
        assert!(diffs[0].contains("expected 3 got 2"), "wrong: {}", diffs[0]);

        // Regs level: always empty
        let diffs_regs = diff_cycles(&expected, &got, Level::Regs);
        assert!(diffs_regs.is_empty(), "regs should be empty");
    }

    #[test]
    fn e02_13_diff_cycles_full_ok() {
        let expected = vec![
            CycleRec { addr: Some(0x8000), val: Some(0xEA), kind: "rrr".to_string() },
            CycleRec { addr: None, val: None, kind: "iii".to_string() },
        ];
        let got = vec![
            Cycle { addr: 0x8000, val: 0xEA, kind: CycleKind::Read },
            Cycle { addr: 0, val: 0, kind: CycleKind::Idle },
        ];
        let diffs = diff_cycles(&expected, &got, Level::Full);
        assert!(diffs.is_empty(), "expected no diffs, got: {:?}", diffs);
    }

    #[test]
    fn e02_13_diff_cycles_full_mismatch() {
        let expected = vec![
            CycleRec { addr: Some(0x1000), val: Some(0x42), kind: "www".to_string() },
        ];
        let got = vec![
            Cycle { addr: 0x1000, val: 0x41, kind: CycleKind::Read },
        ];
        let diffs = diff_cycles(&expected, &got, Level::Full);
        assert_eq!(diffs.len(), 1, "expected 1 mismatch");
        assert!(diffs[0].contains("cycle 0"), "wrong line: {}", diffs[0]);
    }

    #[test]
    fn e02_13_diff_cycles_max_5() {
        let mut expected = Vec::new();
        let mut got = Vec::new();
        for i in 0..10 {
            expected.push(CycleRec { addr: Some(0x1000 + i as u32), val: Some(i as u8), kind: "www".to_string() });
            got.push(Cycle { addr: 0x2000, val: 0xFF, kind: CycleKind::Read });
        }
        let diffs = diff_cycles(&expected, &got, Level::Full);
        assert_eq!(diffs.len(), 5, "expected exactly 5 lines, got {}", diffs.len());
    }

    #[test]
    fn e02_13_diff_cycles_dead_cycle_ignored() {
        let expected = vec![
            CycleRec { addr: Some(0x8000), val: Some(0xDB), kind: "rrr".to_string() },
            CycleRec { addr: None, val: None, kind: "iii".to_string() },
            CycleRec { addr: None, val: None, kind: "iii".to_string() },
            CycleRec { addr: None, val: None, kind: "--------".to_string() },
        ];
        let got = vec![
            Cycle { addr: 0x8000, val: 0xDB, kind: CycleKind::Read },
            Cycle { addr: 0, val: 0, kind: CycleKind::Idle },
            Cycle { addr: 0, val: 0, kind: CycleKind::Idle },
        ];
        let diffs_full = diff_cycles(&expected, &got, Level::Full);
        assert!(diffs_full.is_empty(), "full should be empty, got: {:?}", diffs_full);
        let diffs_count = diff_cycles(&expected, &got, Level::Count);
        assert!(diffs_count.is_empty(), "count should be empty, got: {:?}", diffs_count);
    }

    #[test]
    fn e02_13_apply_initial_emulation_s() {
        // Emulation mode: S high byte forced to 0x01
        let mut st = make_cpu_state();
        st.e = true;
        st.s = 0x2B3E;
        let mut cpu = Cpu::new();
        let mut bus = FlatBus::new();
        apply_initial(&st, &mut cpu, &mut bus);
        assert_eq!(cpu.s, 0x013E, "emulation S should be 0x013E");

        // Native mode: S unchanged
        st.e = false;
        let mut cpu2 = Cpu::new();
        apply_initial(&st, &mut cpu2, &mut bus);
        assert_eq!(cpu2.s, 0x2B3E, "native S should be 0x2B3E");
    }

    #[test]
    fn e02_13_block_move_opcodes() {
        assert!(is_block_move(0x54), "MVN should be block move");
        assert!(is_block_move(0x44), "MVP should be block move");
        assert!(!is_block_move(0x4C), "JMP should not be block move");
        assert!(!is_block_move(0xEA), "NOP should not be block move");
    }
}
