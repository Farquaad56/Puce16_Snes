use puce16_core::ppu::bg::{bpp, priority_order, Pix, Pick, LAYER_OBJ, LAYER_BACKDROP};

#[test]
fn e05_01_bpp_table() {
    // mode 0: [2,2,2,2]
    assert_eq!(bpp(0, 0), 2);
    assert_eq!(bpp(0, 1), 2);
    assert_eq!(bpp(0, 2), 2);
    assert_eq!(bpp(0, 3), 2);

    // mode 1: [4,4,2,0]
    assert_eq!(bpp(1, 0), 4);
    assert_eq!(bpp(1, 1), 4);
    assert_eq!(bpp(1, 2), 2);
    assert_eq!(bpp(1, 3), 0);

    // mode 2: [4,4,0,0]
    assert_eq!(bpp(2, 0), 4);
    assert_eq!(bpp(2, 1), 4);
    assert_eq!(bpp(2, 2), 0);
    assert_eq!(bpp(2, 3), 0);

    // mode 3: [8,4,0,0]
    assert_eq!(bpp(3, 0), 8);
    assert_eq!(bpp(3, 1), 4);
    assert_eq!(bpp(3, 2), 0);
    assert_eq!(bpp(3, 3), 0);

    // mode 4: [8,2,0,0]
    assert_eq!(bpp(4, 0), 8);
    assert_eq!(bpp(4, 1), 2);
    assert_eq!(bpp(4, 2), 0);
    assert_eq!(bpp(4, 3), 0);

    // mode 5: [4,2,0,0]
    assert_eq!(bpp(5, 0), 4);
    assert_eq!(bpp(5, 1), 2);
    assert_eq!(bpp(5, 2), 0);
    assert_eq!(bpp(5, 3), 0);

    // mode 6: [4,0,0,0]
    assert_eq!(bpp(6, 0), 4);
    assert_eq!(bpp(6, 1), 0);
    assert_eq!(bpp(6, 2), 0);
    assert_eq!(bpp(6, 3), 0);

    // mode 7: [8,0,0,0]
    assert_eq!(bpp(7, 0), 8);
    assert_eq!(bpp(7, 1), 0);
    assert_eq!(bpp(7, 2), 0);
    assert_eq!(bpp(7, 3), 0);

    // out of range
    assert_eq!(bpp(8, 0), 0);
    assert_eq!(bpp(0, 4), 0);
}

#[test]
fn e05_01_order_mode0() {
    let expected = [
        (4, 3), // S3
        (0, 1), // 1H
        (1, 1), // 2H
        (4, 2), // S2
        (0, 0), // 1L
        (1, 0), // 2L
        (4, 1), // S1
        (2, 1), // 3H
        (3, 1), // 4H
        (4, 0), // S0
        (2, 0), // 3L
        (3, 0), // 4L
    ];
    assert_eq!(priority_order(0, false, false), &expected);
}

#[test]
fn e05_01_order_mode1() {
    let expected_no_bg3 = [
        (4, 3), // S3
        (0, 1), // 1H
        (1, 1), // 2H
        (4, 2), // S2
        (0, 0), // 1L
        (1, 0), // 2L
        (4, 1), // S1
        (2, 1), // 3H
        (4, 0), // S0
        (2, 0), // 3L
    ];
    assert_eq!(priority_order(1, false, false), &expected_no_bg3);

    let expected_bg3 = [
        (2, 1), // 3H
        (4, 3), // S3
        (0, 1), // 1H
        (1, 1), // 2H
        (4, 2), // S2
        (0, 0), // 1L
        (1, 0), // 2L
        (4, 1), // S1
        (4, 0), // S0
        (2, 0), // 3L
    ];
    assert_eq!(priority_order(1, true, false), &expected_bg3);
}

#[test]
fn e05_01_order_modes_2_to_5() {
    let expected = [
        (4, 3), // S3
        (0, 1), // 1H
        (4, 2), // S2
        (1, 1), // 2H
        (4, 1), // S1
        (0, 0), // 1L
        (4, 0), // S0
        (1, 0), // 2L
    ];
    for m in 2..=5 {
        assert_eq!(priority_order(m, false, false), &expected);
        assert_eq!(priority_order(m, true, false), &expected);
    }
}

#[test]
fn e05_01_order_mode6() {
    let expected = [
        (4, 3), // S3
        (0, 1), // 1H
        (4, 2), // S2
        (4, 1), // S1
        (0, 0), // 1L
        (4, 0), // S0
    ];
    assert_eq!(priority_order(6, false, false), &expected);
}

#[test]
fn e05_01_order_mode7() {
    let expected_no_extbg = [
        (4, 3), // S3
        (4, 2), // S2
        (4, 1), // S1
        (0, 0), // 1
        (4, 0), // S0
    ];
    assert_eq!(priority_order(7, false, false), &expected_no_extbg);

    let expected_extbg = [
        (4, 3), // S3
        (4, 2), // S2
        (1, 1), // 2H
        (4, 1), // S1
        (0, 0), // 1
        (4, 0), // S0
        (1, 0), // 2L
    ];
    assert_eq!(priority_order(7, false, true), &expected_extbg);
}

#[test]
fn e05_01_bg3_priority_only_mode1() {
    assert_eq!(priority_order(0, true, false), priority_order(0, false, false));
    assert_eq!(priority_order(2, true, false), priority_order(2, false, false));
    assert_ne!(priority_order(1, true, false), priority_order(1, false, false));
}

#[test]
fn e05_01_types() {
    let p = Pix { rgb: 1, prio: 2, obj_pal: 3 };
    let q = Pix { rgb: 1, prio: 2, obj_pal: 3 };
    assert_eq!(p, q);

    let pk = Pick { rgb: 1, layer: 5, obj_pal: 0 };
    let pk2 = Pick { rgb: 1, layer: 5, obj_pal: 0 };
    assert_eq!(pk, pk2);

    // Copy trait check - compile-time only
    let _p_copy = p;
    let _pk_copy = pk;

    assert_eq!(LAYER_OBJ, 4);
    assert_eq!(LAYER_BACKDROP, 5);
}
