#!/usr/bin/env python3
"""Check docs/specs/data/gauss_table.tsv against the invariants of spec 07b section 8.
usage: python docs/tools/check_gauss.py [path]   (exit 0 = OK, 1 = problem)
"""
import sys

path = sys.argv[1] if len(sys.argv) > 1 else "docs/specs/data/gauss_table.tsv"
lines = open(path, encoding="utf-8").read().splitlines()
errs = []
if not lines or lines[0].split("\t") != ["index", "value"]:
    errs.append("first line must be: index<TAB>value")
rows = [l.split("\t") for l in lines[1:] if l.strip()]
if len(rows) != 512:
    errs.append("need 512 data rows, got %d" % len(rows))
vals = []
for i, r in enumerate(rows):
    if len(r) != 2 or not r[0].isdigit() or not r[1].isdigit() or int(r[0]) != i:
        errs.append("bad row %d: %r" % (i, r))
        break
    vals.append(int(r[1]))
if not errs:
    if vals[0] != 0:
        errs.append("GAUSS[0] must be 0")
    if vals[511] != 1305:
        errs.append("GAUSS[511] must be 1305")
    for i in range(1, 512):
        if vals[i] < vals[i - 1]:
            errs.append("not non-decreasing at %d" % i)
            break
    for f in range(256):
        s = vals[255 - f] + vals[511 - f] + vals[256 + f] + vals[f]
        if not 2045 <= s <= 2051:
            errs.append("tap sum out of [2045,2051] at f=%d: %d" % (f, s))
            break
if errs:
    print("\n".join(errs))
    sys.exit(1)
print("gauss table OK")
