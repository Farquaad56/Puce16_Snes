use crate::spc_test::oracle::{Case, CycleRec, SpcState};
use puce16_core::apu::spc700::testbus::{CycleKind, FlatSmpBus};
use puce16_core::apu::spc700::Spc700;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum Level { Regs, Count, Full }

fn fmt_u8(v: u8) -> String { format!("0x{:02X}", v) }
fn fmt_u16(v: u16) -> String { format!("0x{:04X}", v) }

pub fn diff_state(expected: &SpcState, cpu: &Spc700, bus: &FlatSmpBus) -> Vec<String> {
    let mut lines = Vec::new();
    if expected.pc != cpu.pc {
        lines.push(format!("pc: expected {} got {}", fmt_u16(expected.pc), fmt_u16(cpu.pc)));
    }
    if expected.a != cpu.a {
        lines.push(format!("a: expected {} got {}", fmt_u8(expected.a), fmt_u8(cpu.a)));
    }
    if expected.x != cpu.x {
        lines.push(format!("x: expected {} got {}", fmt_u8(expected.x), fmt_u8(cpu.x)));
    }
    if expected.y != cpu.y {
        lines.push(format!("y: expected {} got {}", fmt_u8(expected.y), fmt_u8(cpu.y)));
    }
    if expected.sp != cpu.sp {
        lines.push(format!("sp: expected {} got {}", fmt_u8(expected.sp), fmt_u8(cpu.sp)));
    }
    if expected.psw != cpu.psw {
        lines.push(format!("psw: expected {} got {}", fmt_u8(expected.psw), fmt_u8(cpu.psw)));
    }
    for (addr, val) in &expected.ram {
        let got = bus.peek(*addr);
        if *val != got {
            lines.push(format!(
                "ram[{}]: expected {} got {}",
                fmt_u16(*addr), fmt_u8(*val), fmt_u8(got)
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

pub fn diff_cycles(expected: &[CycleRec], got_count: u32, bus: &FlatSmpBus, level: Level) -> Vec<String> {
    if level == Level::Regs {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let expected_len = expected.len() as u32;
    if got_count != expected_len {
        lines.push(format!("cycles: expected {} got {}", expected_len, got_count));
    }
    if level == Level::Count {
        return lines;
    }
    // Full comparison - compare each non-idle cycle
    let log = &bus.log;
    let min_len = std::cmp::min(expected.len(), log.len());
    for i in 0..min_len {
        if lines.len() >= 5 { break; }
        let exp = &expected[i];
        let got = &log[i];
        // Skip idle cycles (no addr/val to compare)
        if exp.kind == "idle" || exp.addr.is_none() { continue; }
        let exp_addr = exp.addr.unwrap_or(0);
        let exp_val = exp.val.unwrap_or(0);
        if got.kind != CycleKind::Read && got.kind != CycleKind::Write { continue; }
        if exp_addr != got.addr || exp_val != got.val {
            lines.push(format!(
                "cycle {}: expected {} {}={} got {} {}={}",
                i, cycle_kind_str(got.kind), fmt_u16(exp_addr), fmt_u8(exp_val),
                cycle_kind_str(got.kind), fmt_u16(got.addr), fmt_u8(got.val)
            ));
        }
    }
    lines.truncate(5);
    lines
}

pub fn run_case(case: &Case, cpu: &mut Spc700, bus: &mut FlatSmpBus, level: Level) -> Vec<String> {
    bus.reset_mem();
    for (addr, val) in &case.initial.ram {
        let mut b = [0u8; 1];
        b[0] = *val;
        bus.poke(*addr, &b);
    }
    cpu.a = case.initial.a;
    cpu.x = case.initial.x;
    cpu.y = case.initial.y;
    cpu.sp = case.initial.sp;
    cpu.pc = case.initial.pc;
    cpu.psw = case.initial.psw;
    cpu.halted = false;
    bus.clear_log();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cpu.step(bus)
    }));

    match result {
        Ok(n) => {
            let mut diffs = diff_state(&case.fin, cpu, bus);
            diffs.extend(diff_cycles(&case.cycles, n, bus, level));
            diffs
        }
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "unknown panic".to_string()
            };
            vec![format!("panic: {}", msg)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(pc: u16, a: u8) -> SpcState {
        SpcState { pc, a, x: 0x78, y: 0x9A, sp: 0xFF, psw: 0x02, ram: Vec::new() }
    }

    fn make_cpu(pc: u16, a: u8) -> Spc700 {
        let mut cpu = Spc700::new();
        cpu.pc = pc;
        cpu.a = a;
        cpu.x = 0x78;
        cpu.y = 0x9A;
        cpu.sp = 0xFF;
        cpu.psw = 0x02;
        cpu
    }

    #[test]
    fn e06_08_diff_state_equal() {
        let st = make_state(0x1234, 0xAB);
        let mut cpu = make_cpu(0x1234, 0xAB);
        let bus = FlatSmpBus::new();
        // Apply initial state to CPU via run_case pattern
        cpu.a = st.a; cpu.x = st.x; cpu.y = st.y; cpu.sp = st.sp; cpu.pc = st.pc; cpu.psw = st.psw;
        let diffs = diff_state(&st, &cpu, &bus);
        assert!(diffs.is_empty(), "expected no diffs, got: {:?}", diffs);
    }

    #[test]
    fn e06_08_diff_state_fields() {
        let expected = make_state(0x5678, 0x12);
        let cpu = make_cpu(0x1234, 0x10); // a=0x10 (expected 0x12), pc different
        let bus = FlatSmpBus::new();
        let diffs = diff_state(&expected, &cpu, &bus);
        assert_eq!(diffs.len(), 2, "expected 2 diffs, got: {:?}", diffs);
        assert!(diffs[0].starts_with("pc:"), "first should be pc diff");
        assert!(diffs.iter().any(|d| d.starts_with("a: expected 0x12 got 0x10")), "missing a diff: {:?}", diffs);
    }

    #[test]
    fn e06_08_diff_state_ram() {
        let mut st = make_state(0x1234, 0xAB);
        st.ram.push((0x1234, 0x41));
        let cpu = make_cpu(0x1234, 0xAB);
        let mut bus = FlatSmpBus::new();
        // Write different value to that address
        let mut b = [0u8; 1];
        b[0] = 0x42;
        bus.poke(0x1234, &b);
        let diffs = diff_state(&st, &cpu, &bus);
        assert_eq!(diffs.len(), 1, "expected 1 ram diff, got: {:?}", diffs);
        assert!(diffs[0].contains("ram[0x1234]"), "wrong line: {}", diffs[0]);
    }

    #[test]
    fn e06_08_diff_cycles_count() {
        let expected = vec![
            CycleRec { addr: Some(0x75F0), val: Some(0xEA), kind: "read".to_string() },
            CycleRec { addr: None, val: None, kind: "idle".to_string() },
            CycleRec { addr: None, val: None, kind: "idle".to_string() },
        ];
        // Count level: expected 3, got 2
        let bus = FlatSmpBus::new();
        let diffs = diff_cycles(&expected, 2, &bus, Level::Count);
        assert_eq!(diffs.len(), 1, "expected 1 count diff");
        assert!(diffs[0].contains("expected 3 got 2"), "wrong: {}", diffs[0]);

        // Regs level: always empty
        let diffs_regs = diff_cycles(&expected, 2, &bus, Level::Regs);
        assert!(diffs_regs.is_empty(), "regs should be empty");
    }

    #[test]
    fn e06_08_run_case_nop_like() {
        // Written when no opcode was implemented (E8 panicked). Since E06-28 every
        // opcode exists, so run_case must NOT report a panic for E8 any more.
        let mut st = make_state(0x75F0, 0x00);
        st.ram.push((0x75F0, 0xE8)); // MOV A,#imm
        st.ram.push((0x75F1, 0x42)); // immediate byte
        let case = Case {
            name: "test".to_string(),
            initial: SpcState { pc: 0x75F0, a: 0x00, x: 0x78, y: 0x9A, sp: 0xFF, psw: 0x02, ram: vec![(0x75F0, 0xE8), (0x75F1, 0x42)] },
            fin: make_state(0x75F2, 0x42),
            cycles: vec![],
        };
        let mut cpu = Spc700::new();
        let mut bus = FlatSmpBus::new();
        let diffs = run_case(&case, &mut cpu, &mut bus, Level::Regs);
        assert!(diffs.iter().all(|d| !d.starts_with("panic:")), "unexpected panic: {:?}", diffs);
    }
}

