"""Replace sbc_decimal_n in crates/puce16-core/src/cpu/alu/sbc.rs by the reference
version from docs/specs/data/bcd_reference.md, and create the E02-27 marker test file
if it does not exist. Usage (repo root): python docs/tools/patch_sbc.py
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SBC = ROOT / "crates/puce16-core/src/cpu/alu/sbc.rs"
REF = ROOT / "docs/specs/data/bcd_reference.md"
TEST = ROOT / "crates/puce16-core/tests/e02_27_oracle_regressions.rs"

MARKER = """// E02-27: oracle regression tests (SBC decimal rewritten from docs/specs/data/bcd_reference.md).
use puce16_core::cpu::testbus::FlatBus;
use puce16_core::cpu::{Bus, Cpu, FLAG_C, FLAG_D};

#[test]
fn e02_27_oracle_clean_or_regressions() {
    assert_eq!(Cpu::new().p, 0x34);
}

#[test]
fn e02_27_sbc_decimal_16bit_borrow_chain() {
    // native, 16-bit A, D=1, C=1: 0x0100 - 0x0001 = 0x0099 (BCD), C stays 1
    let mut cpu = Cpu::new();
    cpu.e = false;
    cpu.p = FLAG_D | FLAG_C;
    cpu.a = 0x0100;
    cpu.pc = 0;
    let mut bus = FlatBus::new();
    for (i, b) in [0xE9u8, 0x01, 0x00].iter().enumerate() {
        bus.write(i as u32, *b);
    }
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x0099);
    assert!(cpu.p & FLAG_C != 0);
}
"""


def main():
    ref = REF.read_text(encoding="utf-8")
    m = re.search(r"(// SBC decimal.*?\nfn sbc_decimal_n.*?\n\}\n)", ref, re.S)
    if not m:
        print("ERROR: reference function not found in bcd_reference.md")
        return 2
    new_fn = m.group(1)
    raw = SBC.read_bytes().decode("utf-8")
    crlf = "\r\n" in raw
    src = raw.replace("\r\n", "\n")
    s = src.find("fn sbc_decimal_n")
    if s < 0:
        print("ERROR: fn sbc_decimal_n not found in sbc.rs")
        return 2
    # include a directly preceding comment block, if any
    while True:
        p = src.rfind("\n", 0, s - 1)
        line = src[p + 1:s - 1] if s > 0 else ""
        if line.lstrip().startswith("//"):
            s = p + 1
        else:
            break
    i = src.index("{", src.find("fn sbc_decimal_n"))
    depth, j = 0, i
    while True:
        if src[j] == "{":
            depth += 1
        elif src[j] == "}":
            depth -= 1
            if depth == 0:
                break
        j += 1
    src = src[:s] + new_fn.rstrip("\n") + src[j + 1:]
    if crlf:
        src = src.replace("\n", "\r\n")
    SBC.write_bytes(src.encode("utf-8"))
    print("sbc.rs: sbc_decimal_n replaced")
    if not TEST.exists():
        TEST.write_bytes((MARKER.replace("\n", "\r\n") if crlf else MARKER).encode("ascii"))
        print("created", TEST.relative_to(ROOT))
    else:
        print("kept existing", TEST.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main())
