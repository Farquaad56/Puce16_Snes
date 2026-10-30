#!/usr/bin/env python3
"""Check the S-DD1 data files and (optionally) the Rust tables against them.
usage: python docs/tools/check_sdd1.py [--rust crates/puce16-core/src/coprocessor/sdd1/tables.rs]
exit 0 = OK, 1 = problem.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import sdd1_ref as ref  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(HERE))
TABLES = os.path.join(ROOT, "docs", "specs", "data", "sdd1_tables.tsv")
VECTORS = os.path.join(ROOT, "docs", "specs", "data", "sdd1_vectors.tsv")

errs = []


def read(path):
    return open(path, encoding="utf-8").read()


# 1. data files must equal what the reference model generates
if read(TABLES) != ref.tables():
    errs.append("sdd1_tables.tsv differs from `python docs/tools/sdd1_ref.py tables`")
if read(VECTORS) != ref.vectors():
    errs.append("sdd1_vectors.tsv differs from `python docs/tools/sdd1_ref.py vectors`")

# 2. optional: Rust tables
if "--rust" in sys.argv:
    path = sys.argv[sys.argv.index("--rust") + 1]
    if not os.path.isfile(path):
        errs.append("missing Rust file %s" % path)
    else:
        src = read(path)
        m = re.search(r"pub const RUNCOUNT:\s*\[u8;\s*256\]\s*=\s*\[(.*?)\];", src, re.S)
        if not m:
            errs.append("RUNCOUNT not found (expected `pub const RUNCOUNT: [u8; 256] = [...];`)")
        else:
            vals = [int(x, 0) for x in re.findall(r"0x[0-9a-fA-F]+|\d+", m.group(1))]
            if vals != ref.RUNCOUNT:
                errs.append("RUNCOUNT differs from the reference table")
        m = re.search(r"pub const EVOLUTION:\s*\[\(u8,\s*u8,\s*u8\);\s*33\]\s*=\s*\[(.*?)\];", src, re.S)
        if not m:
            errs.append("EVOLUTION not found (expected `pub const EVOLUTION: [(u8, u8, u8); 33] = [...];`)")
        else:
            nums = [int(x, 0) for x in re.findall(r"0x[0-9a-fA-F]+|\d+", m.group(1))]
            trip = [tuple(nums[i:i + 3]) for i in range(0, len(nums), 3)]
            if trip != ref.EVOLUTION:
                errs.append("EVOLUTION differs from the reference table")

if errs:
    for e in errs:
        print("CHECK SDD1 FAIL:", e)
    sys.exit(1)
print("CHECK SDD1 OK")
