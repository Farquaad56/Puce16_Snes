# E08-04: impl State for Timing in timing/mod.rs

Add `impl crate::savestate::State for Timing` to the end of the file (before any existing tests).

Save all fields in declaration order. All are covered by hash_into, so save all:
1. line_start (u64)
2. refreshed_this_line (bool)
3. nmi_edge (bool)
4. nmi_line_prev (bool)
5. autoread_start (u64)
6. autoread_end (Option<u64>) - save as bool presence then u64 value if Some

Add #[cfg(test)] mod tests with these tests (prefix e08_04_timing_):
- roundtrip_all_fields: set every saved field to distinct non-zero values, including autoread_end = Some(12345), save, load into fresh Timing::new(), re-save and compare bytes AND hash_into equality.
- roundtrip_autoread_none: same but with autoread_end = None (two cases total).
- truncated: for n in {0, len/2, len-1}, loading data[..n] returns Err(StateError::Truncated).
- behaves_identically: two Timing instances (original and restored) fed the same sequence of on_dot calls via a test rig produce identical TimingOut over 2000 dots.

Do NOT run git commands. Do NOT modify any other files.