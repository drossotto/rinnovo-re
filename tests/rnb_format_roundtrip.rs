// tests/rnb_format_roundtrip.rs
#[test]
fn rnb_empty_roundtrip() {
    let mut p = std::env::temp_dir();
    p.push(format!("rnb_empty_{}.rnb", std::process::id()));

    rnb_format::write_empty_rnb(&p).unwrap();
    let (header, dir) = rnb_format::open_rnb(&p).unwrap();

    assert_eq!(&header.magic, b"RNB\0");
    assert_eq!(dir.entry_count, 0);

    let _ = std::fs::remove_file(&p);
}