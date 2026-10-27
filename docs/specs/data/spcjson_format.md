# SPC700 Oracle JSON Format Specification

Format of the SingleStepTests/spc700 oracle files (`tests/oracle/spc700/v1/*.json`).
Reviewed by Claude (gate E06-03) against ALL 256 real files (256000 tests):
every statement below was checked by script, not by sampling.

## 1. Fichiers

- **Location**: `tests/oracle/spc700/v1/*.json` (ignored by git).
- **Naming**: `<opcode>.json`, opcode = 2 lowercase hex digits (`00.json`, `8d.json`, `ff.json`). The file name IS the opcode under test.
- **Count**: 256 files (one per opcode byte), exactly 1000 tests per file (256000 total).
- **Structure**: each file is a top-level JSON array of test objects.
- **Test name**: `"<OP> <IDX>"` where OP = opcode in UPPERCASE 2-digit hex and IDX = test index in UPPERCASE 4-digit hex, sequential `0000`..`03E7` (`"00 0000"`, `"0F 0001"`, `"E4 03E7"`). IDX is only a counter, it is NOT an operand.

## 2. Cles initial et final

```json
{ "name": "<OP> <IDX>", "initial": { ... }, "final": { ... }, "cycles": [ ... ] }
```

State object (both `initial` and `final`; always all 7 keys; key order differs between the two, irrelevant in JSON):

| Key | JSON type | Description |
|-----|-----------|-------------|
| `pc` | integer | program counter, 0-65535 |
| `a`, `x`, `y` | integer | 8-bit registers |
| `sp` | integer | 8-bit stack pointer (see 4) |
| `psw` | integer | 8-bit status word (see 4) |
| `ram` | array | sparse memory: list of `[address, value]` pairs |

Rust loader note: the JSON key `final` is a Rust keyword; use `#[serde(rename = "final")]` on the field (`fin`).

### RAM

- `ram` = `[[addr, val], ...]`, addr 0-65535, val 0-255. Sorted by address, no duplicate address.
- It lists only the addresses relevant to the test: the opcode/operand bytes, every address the instruction reads or writes, and vectors. All other addresses are unspecified (use 0).
- `initial.ram` contains the opcode byte at `initial.pc`.
- `final.ram` = `initial.ram` plus every write of `cycles` applied (all initial addresses are kept). Check `final.ram` against the memory of the bus after the step, for the addresses listed in `final.ram`.
- RAM is FLAT: 5154 reads in `$F0-$FF` were checked, every one returns the plain RAM value. No I/O registers, no timers, no IPL ROM. The test bus must be flat (`FlatSmpBus`).

## 3. Cycles

`cycles` = array of 3-element arrays `[address, value, kind]` (in this order), one per bus cycle. `kind` is a lowercase string:

| kind | address | value | Description |
|------|---------|-------|-------------|
| `"read"` | integer | integer OR `null` | read; `null` value = dummy read (the value is not meaningful: never compare it) |
| `"write"` | integer | integer | write |
| `"wait"` | `null` | `null` | internal cycle, no bus access |

- The number of elements of `cycles` is the instruction length in cycles. Verified: the first cycle is always a `read` of `initial.pc` returning the opcode.
- `null` read values exist in 60 opcodes (e.g. `00` NOP, `0F` BRK, `1D` DEC X). Rust loader: `Option<u16>` / `Option<u8>` for addr and val; the cycle can be deserialized as a struct with fields in the order `addr, val, kind` (serde accepts a JSON array for a derived struct) or as a tuple.
- Branch opcodes have a variable cycle count: base cycles when not taken, base + 2 when taken (see 6).

### Example: opcode `91` = TCALL 9 (8 cycles)

```json
"cycles": [
  [51742, 145, "read"],   // opcode fetch
  [51743, 12, "read"],    // dummy read, next byte
  [null, null, "wait"],
  [453, 202, "write"],    // push PCH at 0x100+sp (sp was 197)
  [452, 31, "write"],     // push PCL
  [null, null, "wait"],
  [65484, 234, "read"],   // vector low  (0xFFC0 + 2*(15-9))
  [65485, 172, "read"]    // vector high
]
```

### Example: opcode `00` = NOP (2 cycles)

```json
"cycles": [ [30256, 0, "read"], [30257, null, "read"] ]
```

The second cycle is a dummy read (value `null`), not an operand.

## 4. Convention des registres (verifiee)

- PSW bits: `N=0x80 V=0x40 P=0x20 B=0x10 H=0x08 I=0x04 Z=0x02 C=0x01`.
- `P` (0x20) selects the direct page: dp address = `(P ? 0x100 : 0x000) + dp`. Verified (`E4 0000`: psw=235 has P=1, dp operand 0x74 reads address 372 = 0x174).
- Stack: push writes at `0x100 + sp` THEN decrements `sp`; pop increments then reads. So `sp` points to the NEXT FREE slot. Verified (`0F 0000`: sp=107, writes at 363, 362, 361, final sp=104).
- `final.pc` is the pc after the instruction (including branch target or jump).

## 5. Exemple complet

Complete real test (opcode `00` = NOP, file `00.json`, first test):

```json
{
  "name": "00 0000",
  "initial": { "pc": 30256, "a": 56, "x": 78, "y": 127, "sp": 236, "psw": 145,
               "ram": [ [30256, 0] ] },
  "final":   { "a": 56, "x": 78, "y": 127, "sp": 236, "pc": 30257, "psw": 145,
               "ram": [ [30256, 0] ] },
  "cycles": [ [30256, 0, "read"], [30257, null, "read"] ]
}
```

## 6. Decisions de la gate E06-03

1. **Comparison level**: `Level::Count` (final state + number of cycles) is the default for the whole epic. `Level::Full` is NOT used: the journal is consistent (checked: first cycle = fetch, reads match RAM, `final.ram` = initial + writes) but it contains `wait` cycles and dummy reads with `null` value, which have no equivalent in the `Idle`/`Read` log of `FlatSmpBus`, so an index-by-index comparison would give false failures. A later task may add `Full` restricted to non-null `read`/`write` cycles.
2. **TSV vs oracle**: all 228 non-branch opcodes match the oracle, except three that were wrong in the TSV and are now corrected to the oracle: `DA MOVW dp,YA` 4 -> 5 cycles; `EF SLEEP` 3 -> 7; `FF STOP` 2 -> 7. For SLEEP and STOP the oracle does not halt: it executes 7 cycles, leaves all registers and `psw` unchanged and sets `final.pc = pc + 1`. The emulator must reproduce that when run against the oracle. The 28 conditional opcodes (`BPL BMI BVC BVS BCC BCS BNE BEQ`, the 16 `BBS/BBC`, `CBNE dp`, `CBNE dp+X`, `DBNZ dp`, `DBNZ Y`) have `branch_taken_extra = 2` and the oracle shows exactly base and base+2 (both occur in the files). `BRA` is fixed 4 cycles, `DBNZ`/`CBNE` not-taken cases are rare (3-4 per 1000).
3. **Filter convention** (`--filter`): comma-separated list of opcodes in hex, 1 or 2 digits, case-insensitive, no `0x` prefix, no spaces required (`E4,F4`, `e4,0f,0`). A file is kept when its opcode is in the list.
4. **`opcode_of_file(name)`**: takes a file name or a path, uses the last path component; it must be exactly 2 hex digits (either case) followed by `.json`; returns `Some(u8)`, otherwise `None` (`readme.md`, `LICENSE`, `100.json`, `zz.json` -> `None`).

STATUS: reviewed
