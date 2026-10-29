use puce16_desktop::config::{Config, config_dir_from};
use std::collections::BTreeMap;

#[test]
fn e08_24_defaults() {
    let cfg = Config::default();
    assert_eq!(cfg.scale, 3);
    assert_eq!(cfg.ratio, "square");
    assert_eq!(cfg.filter, "nearest");
    assert_eq!(cfg.volume, 100);
    assert!(!cfg.muted);
    assert!(cfg.block_opposite_dirs);
    assert!(cfg.recent_roms.is_empty());
}

#[test]
fn e08_24_roundtrip() {
    let mut cfg = Config::default();
    cfg.scale = 5;
    cfg.filter = "scanlines".to_string();
    cfg.volume = 40;
    cfg.add_recent("rom_a.sfc");
    cfg.add_recent("rom_b.sfc");
    let mut keymap = BTreeMap::new();
    keymap.insert("A".to_string(), vec!["x".to_string()]);
    cfg.keymap = keymap;

    let json = cfg.to_json();
    let loaded = Config::from_json(&json);
    assert_eq!(loaded, cfg);
}

#[test]
fn e08_24_invalid_json_gives_defaults() {
    let cfg1 = Config::from_json("not json");
    let cfg2 = Config::from_json("");
    let default_cfg = Config::default();
    assert_eq!(cfg1, default_cfg);
    assert_eq!(cfg2, default_cfg);
}

#[test]
fn e08_24_partial_json() {
    let cfg = Config::from_json("{\"scale\": 6}");
    assert_eq!(cfg.scale, 6);
    assert_eq!(cfg.ratio, "square");
    assert_eq!(cfg.filter, "nearest");
    assert_eq!(cfg.volume, 100);
}

#[test]
fn e08_24_sanitize() {
    let mut cfg = Config::default();
    cfg.scale = 0;
    cfg.sanitize();
    assert_eq!(cfg.scale, 3); // out of range -> default

    cfg.scale = 99;
    cfg.sanitize();
    assert_eq!(cfg.scale, 3); // out of range -> default

    cfg.volume = 250;
    cfg.sanitize();
    assert_eq!(cfg.volume, 100);

    cfg.ratio = "weird".to_string();
    cfg.sanitize();
    assert_eq!(cfg.ratio, "square");

    cfg.filter = "x".to_string();
    cfg.sanitize();
    assert_eq!(cfg.filter, "nearest");

    // 12 recent ROMs -> truncate to 8
    let mut roms = Vec::new();
    for i in 0..12 {
        roms.push(format!("rom_{}.sfc", i));
    }
    cfg.recent_roms = roms;
    cfg.sanitize();
    assert_eq!(cfg.recent_roms.len(), 8);
}

#[test]
fn e08_24_add_recent() {
    let mut cfg = Config::default();
    cfg.add_recent("a");
    cfg.add_recent("b");
    cfg.add_recent("a"); // duplicate moves to front
    assert_eq!(cfg.recent_roms, vec!["a", "b"]);

    // 10 distinct additions -> length 8, last added at front
    let mut cfg2 = Config::default();
    for i in 0..10 {
        cfg2.add_recent(&format!("rom_{}", i));
    }
    assert_eq!(cfg2.recent_roms.len(), 8);
    assert_eq!(cfg2.recent_roms[0], "rom_9");
}

#[test]
fn e08_24_config_dir() {
    let p1 = config_dir_from(Some("C:/Users/x/AppData/Roaming"), None).unwrap();
    assert!(p1.to_string_lossy().contains("Puce16"));

    let p2 = config_dir_from(None, Some("/home/x")).unwrap();
    assert!(p2.to_string_lossy().contains(".config/puce16") || p2.to_string_lossy().contains(".config\\puce16"));

    // Empty appdata falls through to home
    let p3 = config_dir_from(Some(""), Some("/home/x")).unwrap();
    assert!(p3.to_string_lossy().contains(".config/puce16") || p3.to_string_lossy().contains(".config\\puce16"));

    let p4 = config_dir_from(None, None);
    assert!(p4.is_none());
}

#[test]
fn e08_24_save_load_in_temp() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let path = dir.path().join("config.json");

    let mut cfg = Config::default();
    cfg.scale = 5;
    cfg.volume = 40;

    // Write JSON directly to file (simulating save)
    fs::write(&path, cfg.to_json()).unwrap();

    // Read back and parse
    let text = fs::read_to_string(&path).unwrap();
    let loaded = Config::from_json(&text);
    assert_eq!(loaded.scale, 5);
    assert_eq!(loaded.volume, 40);
}
