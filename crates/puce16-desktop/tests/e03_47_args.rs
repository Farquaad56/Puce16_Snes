use puce16_desktop::app::rom_arg;

#[test]
fn e03_47_rom_arg() {
    assert_eq!(rom_arg(&["puce16-desktop".into()]), None);
    assert_eq!(rom_arg(&["puce16-desktop".into(), "game.sfc".into()]), Some("game.sfc"));
    assert_eq!(rom_arg(&["x".into(), "--foo".into(), "a.smc".into()]), Some("a.smc"));
    assert_eq!(rom_arg(&["x".into(), "-v".into()]), None);
}

#[test]
fn e03_47_rom_arg_first_wins() {
    assert_eq!(rom_arg(&["x".into(), "a.sfc".into(), "b.sfc".into()]), Some("a.sfc"));
}
