// tests/rnb_format_roundtrip.rs

#[test]
fn rnb_minimal_roundtrip() {
    let mut p = std::env::temp_dir();
    p.push(format!("rnb_minimal_{}.rnb", std::process::id()));

    rnb_format::write_empty_rnb(&p).unwrap();
    let f = rnb_format::open_rnb(&p).unwrap();

    assert_eq!(&f.header.magic, b"RNB\0");
    assert_eq!(f.directory.entries.len(), 1);

    let e = &f.directory.entries[0];
    assert_eq!(e.segment_id, 1);
    assert_eq!(e.segment_type, rnb_format::SegmentType::Manifest.as_u32());
    assert!(e.length > 0);

    // Manifest invariants
    assert_eq!(f.manifest.required_segments, vec![rnb_format::SegmentType::Manifest]);
    assert!(f.manifest.max_chunk_bytes > 0);

    let _ = std::fs::remove_file(&p);
}