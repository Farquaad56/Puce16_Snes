# E08-04: impl State for Mmio in bus/mmio.rs

Add `impl crate::savestate::State for Mmio` to the end of the file (before any existing tests).

Save all fields in declaration order. All are covered by hash_into, so save all:
1. nmitimen (u8)
2. wrio (u8)
3. htime (u16)
4. vtime (u16)
5. hdmaen (u8)
6. memsel (bool)
7. rdnmi_flag (bool)
8. timeup_flag (bool)
9. irq_line (bool)
10. joy ([u16; 4]) - use slice_u16
11. wrmpya (u8)
12. wrdiv (u16)
13. rddiv (u16)
14. rdmpy (u16)
15. math_pending (Option<MathPending>) - save as u8 index: 0=None, 1=Some; if Some, save done_at(u64), rddiv(u16), rdmpy(u16). On load, unknown index -> StateError::Mismatch("math_pending").

Add #[cfg(test)] mod tests with these tests (prefix e08_04_mmio_):
- roundtrip_all_fields: set every saved field to distinct non-zero values, save, load into fresh Mmio::new(), re-save and compare bytes AND hash_into equality.
- truncated: for n in {0, len/2, len-1}, loading data[..n] returns Err(StateError::Truncated).

Do NOT run git commands. Do NOT modify any other files.