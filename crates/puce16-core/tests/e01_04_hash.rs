use puce16_core::hash::Fnv64;

#[test]
fn e01_04_empty() {
    assert_eq!(Fnv64::new().finish(), 0xcbf29ce484222325);
}

#[test]
fn e01_04_vector_a() {
    let mut h = Fnv64::new();
    h.write(b"a");
    assert_eq!(h.finish(), 0xaf63dc4c8601ec8c);
}

#[test]
fn e01_04_vector_foobar() {
    let mut h = Fnv64::new();
    h.write(b"foobar");
    assert_eq!(h.finish(), 0x85944171f73967e8);
}

#[test]
fn e01_04_u64_le() {
    let mut a = Fnv64::new();
    a.write_u64(1);
    let mut b = Fnv64::new();
    b.write(&[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(a.finish(), b.finish());
    assert_eq!(a.finish(), 0x89cd31291d2aefa4);
}

#[test]
fn e01_04_incremental() {
    let mut a = Fnv64::new();
    a.write(b"foo");
    a.write(b"bar");
    let mut b = Fnv64::new();
    b.write(b"foobar");
    assert_eq!(a.finish(), b.finish());
}
