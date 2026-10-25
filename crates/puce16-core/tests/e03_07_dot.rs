use puce16_core::ppu::dot::{line_type, line_dots, line_clocks, dot_clocks, dot_start_clock, LineType};
use puce16_core::region::Region;
use puce16_core::consts::{VISIBLE_CLOCK_FIRST, VISIBLE_CLOCK_END};

#[test]
fn e03_07_line_type() {
    assert_eq!(line_type(Region::Ntsc, false, true, 240), LineType::Short);
    assert_eq!(line_type(Region::Ntsc, false, false, 240), LineType::Normal);
    assert_eq!(line_type(Region::Ntsc, true, true, 240), LineType::Normal);
    assert_eq!(line_type(Region::Ntsc, false, true, 239), LineType::Normal);
    assert_eq!(line_type(Region::Pal, true, true, 311), LineType::Long);
    assert_eq!(line_type(Region::Pal, true, false, 311), LineType::Normal);
    assert_eq!(line_type(Region::Pal, false, true, 311), LineType::Normal);
    assert_eq!(line_type(Region::Pal, false, true, 240), LineType::Normal);
}

#[test]
fn e03_07_dots_clocks_sums() {
    assert_eq!(line_dots(LineType::Normal), 340);
    assert_eq!(line_dots(LineType::Short), 340);
    assert_eq!(line_dots(LineType::Long), 341);

    assert_eq!(line_clocks(LineType::Normal), 1364);
    assert_eq!(line_clocks(LineType::Short), 1360);
    assert_eq!(line_clocks(LineType::Long), 1368);

    for lt in [LineType::Normal, LineType::Short, LineType::Long] {
        let mut sum = 0u32;
        for d in 0..line_dots(lt) {
            sum += dot_clocks(lt, d);
        }
        assert_eq!(sum, line_clocks(lt), "dot clock sum mismatch for {:?}", lt);
    }
}

#[test]
fn e03_07_five_clock_dots() {
    // Normal: 5 clocks at dots 323, 324, 327, 328; 4 elsewhere
    assert_eq!(dot_clocks(LineType::Normal, 323), 5);
    assert_eq!(dot_clocks(LineType::Normal, 324), 5);
    assert_eq!(dot_clocks(LineType::Normal, 327), 5);
    assert_eq!(dot_clocks(LineType::Normal, 328), 5);
    assert_eq!(dot_clocks(LineType::Normal, 0), 4);
    assert_eq!(dot_clocks(LineType::Normal, 1), 4);
    assert_eq!(dot_clocks(LineType::Normal, 322), 4);
    assert_eq!(dot_clocks(LineType::Normal, 325), 4);
    assert_eq!(dot_clocks(LineType::Normal, 326), 4);
    assert_eq!(dot_clocks(LineType::Normal, 329), 4);
    assert_eq!(dot_clocks(LineType::Normal, 339), 4);

    // Short: all 4 clocks
    for d in [0, 1, 322, 323, 324, 325, 326, 327, 328, 329, 339] {
        assert_eq!(dot_clocks(LineType::Short, d), 4);
    }

    // Long: same as Normal for the five-clock dots, plus dot 340 is 4 clocks
    assert_eq!(dot_clocks(LineType::Long, 323), 5);
    assert_eq!(dot_clocks(LineType::Long, 324), 5);
    assert_eq!(dot_clocks(LineType::Long, 327), 5);
    assert_eq!(dot_clocks(LineType::Long, 328), 5);
    assert_eq!(dot_clocks(LineType::Long, 340), 4);
}

#[test]
fn e03_07_start_clock_normal() {
    let lt = LineType::Normal;
    assert_eq!(dot_start_clock(lt, 0), 0);
    assert_eq!(dot_start_clock(lt, 1), 4);
    assert_eq!(dot_start_clock(lt, 22), 88);
    assert_eq!(dot_start_clock(lt, 274), 1096);
    assert_eq!(dot_start_clock(lt, 323), 1292);
    assert_eq!(dot_start_clock(lt, 324), 1297);
    assert_eq!(dot_start_clock(lt, 325), 1302);
    assert_eq!(dot_start_clock(lt, 326), 1306);
    assert_eq!(dot_start_clock(lt, 327), 1310);
    assert_eq!(dot_start_clock(lt, 328), 1315);
    assert_eq!(dot_start_clock(lt, 329), 1320);
    assert_eq!(dot_start_clock(lt, 340), 1364);
}

#[test]
fn e03_07_start_clock_short_long() {
    let lt = LineType::Short;
    assert_eq!(dot_start_clock(lt, 323), 1292);
    assert_eq!(dot_start_clock(lt, 324), 1296);
    assert_eq!(dot_start_clock(lt, 340), 1360);

    let lt = LineType::Long;
    assert_eq!(dot_start_clock(lt, 323), 1292);
    assert_eq!(dot_start_clock(lt, 329), 1320);
    assert_eq!(dot_start_clock(lt, 340), 1364);
    assert_eq!(dot_start_clock(lt, 341), 1368);
}

#[test]
fn e03_07_visible_window() {
    let lt = LineType::Normal;
    assert_eq!(dot_start_clock(lt, 22), VISIBLE_CLOCK_FIRST);
    assert_eq!(dot_start_clock(lt, 278), VISIBLE_CLOCK_END);
}

#[test]
#[should_panic]
fn e03_07_panic_oob() {
    dot_clocks(LineType::Normal, 340);
}
