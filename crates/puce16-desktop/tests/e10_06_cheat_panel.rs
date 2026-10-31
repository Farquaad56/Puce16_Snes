//! E10-06: cheat panel model tests (pure, no egui).

use puce16_core::cheat::{parse_cheat_file, ParCode};
use puce16_desktop::cheat_panel::{check_codes, CheatPanel};

fn pc(addr: u32, value: u8) -> ParCode {
    ParCode { addr, value }
}

#[test]
fn e10_06_check_codes_valid_separators() {
    let v = check_codes("7E0001AA, 7E0002BB 7E0003CC\n7E0004DD").unwrap();
    assert_eq!(v, vec![pc(0x7E0001, 0xAA), pc(0x7E0002, 0xBB), pc(0x7E0003, 0xCC), pc(0x7E0004, 0xDD)]);
}

#[test]
fn e10_06_check_codes_errors_in_french() {
    assert!(check_codes("").unwrap_err().contains("au moins un code"));
    assert!(check_codes("  \n ").unwrap_err().contains("au moins un code"));
    let e = check_codes("7E0001A").unwrap_err();
    assert!(e.contains("7E0001A") && e.contains("7 chiffres au lieu de 8"));
    assert!(check_codes("7E0001GA").unwrap_err().contains("'G'"));
    assert!(check_codes("DF4B-7D6E").unwrap_err().contains("Game Genie"));
}

#[test]
fn e10_06_add_valid_entry_is_active() {
    let mut panel = CheatPanel::new(vec![]);
    assert!(panel.add("Vies", "7E0DBE05").is_ok());
    assert_eq!(panel.entries().len(), 1);
    let entry = &panel.entries()[0];
    assert!(entry.enabled);
    assert_eq!(entry.codes, vec![pc(0x7E0DBE, 0x05)]);
}

#[test]
fn e10_06_add_empty_name_uses_first_code() {
    let mut panel = CheatPanel::new(vec![]);
    assert!(panel.add("", "7e0dbe05 7E0001AA").is_ok());
    assert_eq!(panel.entries()[0].name, "7E0DBE05");
    assert_eq!(panel.entries()[0].codes.len(), 2);
}

#[test]
fn e10_06_add_rejects_bad_name_and_code() {
    let mut panel = CheatPanel::new(vec![]);
    assert!(panel.add("a=b", "7E0001AA").is_err());
    assert!(panel.add("A", "").is_err());
    assert!(panel.add("A", "7E0001A").is_err());
    assert_eq!(panel.entries().len(), 0);
}

#[test]
fn e10_06_toggle_and_active_codes() {
    let mut panel = CheatPanel::new(vec![]);
    panel.add("A", "7E0001AA").unwrap();
    panel.add("B", "7E0002BB").unwrap();
    panel.add("C", "7E0003CC").unwrap();
    panel.toggle(1);
    assert_eq!(panel.active_codes(), vec![pc(0x7E0001, 0xAA), pc(0x7E0003, 0xCC)]);
    panel.toggle(1);
    assert_eq!(panel.active_codes().len(), 3);
}

#[test]
fn e10_06_disable_all_and_remove() {
    let mut panel = CheatPanel::new(vec![]);
    panel.add("A", "7E0001AA").unwrap();
    panel.add("B", "7E0002BB").unwrap();
    panel.disable_all();
    assert!(panel.active_codes().is_empty());
    assert_eq!(panel.entries().len(), 2);
    panel.remove(0);
    assert_eq!(panel.entries().len(), 1);
    assert_eq!(panel.entries()[0].name, "B");
    panel.remove(99);
    panel.toggle(99);
    assert_eq!(panel.entries().len(), 1);
}

#[test]
fn e10_06_new_keeps_loaded_entries() {
    let entries = parse_cheat_file("+ A = 7E0001AA\n- B = 7E0002BB\n").unwrap();
    let panel = CheatPanel::new(entries);
    assert_eq!(panel.entries().len(), 2);
    assert_eq!(panel.active_codes(), vec![pc(0x7E0001, 0xAA)]);
}
