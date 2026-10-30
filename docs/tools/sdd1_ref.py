#!/usr/bin/env python3
"""Reference model of the S-DD1 decompressor (Python), used to generate the golden vectors of
docs/specs/data/sdd1_vectors.tsv and the tables of docs/specs/data/sdd1_tables.tsv.

The algorithm is the one reverse-engineered by Andreas Naive (public domain), as implemented in
the bsnes source (sfc/coprocessor/sdd1/decompressor.cpp). This file is an independent transcription
(NOT a run of that C++ code): the vectors are only as good as the transcription, and the
final proof is the user's test with real S-DD1 games (gate).

usage:
  python docs/tools/sdd1_ref.py tables  > docs/specs/data/sdd1_tables.tsv
  python docs/tools/sdd1_ref.py vectors > docs/specs/data/sdd1_vectors.tsv
  python docs/tools/sdd1_ref.py check            (self-test, exit 0 = OK)
"""
import sys

# Golomb run-count table as published (256 entries), used to cross-check the formula.
PUBLISHED_RUNCOUNT = """
00 00 01 00 03 01 02 00 07 03 05 01 06 02 04 00
0f 07 0b 03 0d 05 09 01 0e 06 0a 02 0c 04 08 00
1f 0f 17 07 1b 0b 13 03 1d 0d 15 05 19 09 11 01
1e 0e 16 06 1a 0a 12 02 1c 0c 14 04 18 08 10 00
3f 1f 2f 0f 37 17 27 07 3b 1b 2b 0b 33 13 23 03
3d 1d 2d 0d 35 15 25 05 39 19 29 09 31 11 21 01
3e 1e 2e 0e 36 16 26 06 3a 1a 2a 0a 32 12 22 02
3c 1c 2c 0c 34 14 24 04 38 18 28 08 30 10 20 00
7f 3f 5f 1f 6f 2f 4f 0f 77 37 57 17 67 27 47 07
7b 3b 5b 1b 6b 2b 4b 0b 73 33 53 13 63 23 43 03
7d 3d 5d 1d 6d 2d 4d 0d 75 35 55 15 65 25 45 05
79 39 59 19 69 29 49 09 71 31 51 11 61 21 41 01
7e 3e 5e 1e 6e 2e 4e 0e 76 36 56 16 66 26 46 06
7a 3a 5a 1a 6a 2a 4a 0a 72 32 52 12 62 22 42 02
7c 3c 5c 1c 6c 2c 4c 0c 74 34 54 14 64 24 44 04
78 38 58 18 68 28 48 08 70 30 50 10 60 20 40 00
"""

# Probability estimation: state -> (code_number, next_if_mps, next_if_lps)
EVOLUTION = [
    (0, 25, 25), (0, 2, 1), (0, 3, 1), (0, 4, 2), (0, 5, 3), (1, 6, 4), (1, 7, 5), (1, 8, 6),
    (1, 9, 7), (2, 10, 8), (2, 11, 9), (2, 12, 10), (2, 13, 11), (3, 14, 12), (3, 15, 13),
    (3, 16, 14), (3, 17, 15), (4, 18, 16), (4, 19, 17), (5, 20, 18), (5, 21, 19), (6, 22, 20),
    (6, 23, 21), (7, 24, 22), (7, 24, 23), (0, 26, 1), (1, 27, 2), (2, 28, 4), (3, 29, 8),
    (4, 30, 12), (5, 31, 16), (6, 32, 18), (7, 24, 22),
]


def bitrev(x, n):
    r = 0
    for _ in range(n):
        r = (r << 1) | (x & 1)
        x >>= 1
    return r


def make_runcount():
    t = [0] * 256
    for idx in range(2, 256):
        n = idx.bit_length() - 1          # idx in [2^n, 2^(n+1))
        t[idx] = ((1 << n) - 1) - bitrev(idx - (1 << n), n)
    return t


RUNCOUNT = make_runcount()
_pub = [int(x, 16) for x in PUBLISHED_RUNCOUNT.split()]
assert _pub == RUNCOUNT, "run-count formula does not match the published table"


class Decompressor:
    def __init__(self, data, offset=0):
        self.data = data
        self.off = offset
        self.bit_count = 4                       # input manager
        self.mps_count = [0] * 8                 # bits generators
        self.lps_index = [0] * 8
        self.status = [0] * 32                   # probability estimation
        self.mps = [0] * 32
        h = self.rd(offset)
        self.bitplanes_info = h & 0xC0           # context model
        self.context_info = h & 0x30
        self.bit_number = 0
        self.prev = [0] * 8
        self.cur_bp = {0x00: 1, 0x40: 7, 0x80: 3, 0xC0: 0}[self.bitplanes_info]
        self.r0 = 0x01                           # output logic
        self.r1 = self.r2 = 0

    def rd(self, a):
        return self.data[a % len(self.data)]

    # input manager
    def code_word(self, code_length):
        cw = (self.rd(self.off) << self.bit_count) & 0xFF
        self.bit_count += 1
        if cw & 0x80:
            cw |= self.rd(self.off + 1) >> (9 - self.bit_count)
            self.bit_count += code_length
        if self.bit_count & 0x08:
            self.off += 1
            self.bit_count &= 0x07
        return cw & 0xFF

    # golomb-code decoder
    def run_count(self, code_number):
        cw = self.code_word(code_number)
        if cw & 0x80:
            return RUNCOUNT[cw >> (code_number ^ 0x07)], 1
        return 1 << code_number, 0

    # bits generator
    def bg_bit(self, n):
        if not (self.mps_count[n] or self.lps_index[n]):
            self.mps_count[n], self.lps_index[n] = self.run_count(n)
        if self.mps_count[n]:
            bit = 0
            self.mps_count[n] -= 1
        else:
            bit = 1
            self.lps_index[n] = 0
        end_of_run = not (self.mps_count[n] or self.lps_index[n])
        return bit, end_of_run

    # probability estimation
    def pem_bit(self, ctx):
        st = self.status[ctx]
        cur_mps = self.mps[ctx]
        code_number, next_mps, next_lps = EVOLUTION[st]
        bit, end = self.bg_bit(code_number)
        if end:
            if bit:
                if not (st & 0xFE):
                    self.mps[ctx] ^= 1
                self.status[ctx] = next_lps
            else:
                self.status[ctx] = next_mps
        return bit ^ cur_mps

    # context model
    def cm_bit(self):
        bi = self.bitplanes_info
        if bi == 0x00:
            self.cur_bp ^= 1
        elif bi == 0x40:
            self.cur_bp ^= 1
            if not (self.bit_number & 0x7F):
                self.cur_bp = (self.cur_bp + 2) & 7
        elif bi == 0x80:
            self.cur_bp ^= 1
            if not (self.bit_number & 0x7F):
                self.cur_bp ^= 2
        else:
            self.cur_bp = self.bit_number & 7
        cb = self.prev[self.cur_bp]
        ctx = (self.cur_bp & 1) << 4
        ci = self.context_info
        if ci == 0x00:
            ctx |= ((cb & 0x01C0) >> 5) | (cb & 0x0001)
        elif ci == 0x10:
            ctx |= ((cb & 0x0180) >> 5) | (cb & 0x0001)
        elif ci == 0x20:
            ctx |= ((cb & 0x00C0) >> 5) | (cb & 0x0001)
        else:
            ctx |= ((cb & 0x0180) >> 5) | (cb & 0x0003)
        bit = self.pem_bit(ctx)
        self.prev[self.cur_bp] = ((cb << 1) | bit) & 0xFFFF
        self.bit_number = (self.bit_number + 1) & 0xFF
        return bit

    # output logic
    def read(self):
        if self.bitplanes_info != 0xC0:
            if self.r0 == 0:
                self.r0 = (~self.r0) & 0xFF
                return self.r2
            self.r0, self.r1, self.r2 = 0x80, 0, 0
            while self.r0:
                if self.cm_bit():
                    self.r1 |= self.r0
                if self.cm_bit():
                    self.r2 |= self.r0
                self.r0 >>= 1
            return self.r1
        self.r0, self.r1 = 0x01, 0
        while self.r0 & 0xFF:
            if self.cm_bit():
                self.r1 |= self.r0
            self.r0 = (self.r0 << 1) & 0xFF
        return self.r1


def lcg_bytes(seed, n):
    out = []
    x = seed
    for _ in range(n):
        x = (x * 1664525 + 1013904223) & 0xFFFFFFFF
        out.append(x >> 24)
    return out


def fnv64(bs):
    h = 0xCBF29CE484222325
    for b in bs:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def vectors():
    rows = ["header\tseed\tinput_hex_first32\toutput_hex_first64\tfnv64_first256\tfnv64_first8192"]
    for hdr in range(16):
        seed = 0x1000 + hdr
        data = lcg_bytes(seed, 4096)
        data[0] = (data[0] & 0x0F) | (hdr << 4)
        d = Decompressor(data, 0)
        out = [d.read() for _ in range(8192)]
        rows.append("%02x\t%d\t%s\t%s\t%016x\t%016x" % (
            hdr << 4, seed, bytes(data[:32]).hex(), bytes(out[:64]).hex(), fnv64(out[:256]), fnv64(out)))
    return "\n".join(rows) + "\n"


def tables():
    rows = ["# S-DD1 tables (Andreas Naive). Section RUNCOUNT: index<TAB>value (256 rows). Section EVOLUTION: state<TAB>code_number<TAB>next_if_mps<TAB>next_if_lps (33 rows)",
            "RUNCOUNT"]
    for i, v in enumerate(RUNCOUNT):
        rows.append("%d\t%d" % (i, v))
    rows.append("EVOLUTION")
    for i, (c, m, l) in enumerate(EVOLUTION):
        rows.append("%d\t%d\t%d\t%d" % (i, c, m, l))
    return "\n".join(rows) + "\n"


def selftest():
    assert len(EVOLUTION) == 33 and len(RUNCOUNT) == 256
    # plain G0 stream: header byte 0x00, then bits; just check determinism and header decoding
    a = Decompressor([0x00] * 64)
    b = Decompressor([0x00] * 64)
    assert [a.read() for _ in range(32)] == [b.read() for _ in range(32)]
    # an all-zero stream is an endless run of MPS (= 0): the output is all zero
    assert all(x == 0 for x in [Decompressor([0x00] * 4096).read() for _ in range(1)])
    print("OK")


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    if cmd == "tables":
        sys.stdout.write(tables())
    elif cmd == "vectors":
        sys.stdout.write(vectors())
    else:
        selftest()
