// tests/rnb_format_roundtrip.rs

#[test]
fn rnb_minimal_roundtrip_with_dict() {
    let mut p = std::env::temp_dir();
    p.push(format!("rnb_minimal_dict_{}.rnb", std::process::id()));

    rnb_format::write_empty_rnb(&p).unwrap();
    let f = rnb_format::open_rnb(&p).unwrap();

    assert_eq!(&f.header.magic, b"RNB\0");
    assert!(f.directory.entries.len() >= 1);

    // Manifest exists
    assert_eq!(
        f.directory.entries.iter().filter(|e| e.segment_type == rnb_format::SegmentType::Manifest.as_u32()).count(),
        1
    );

    // Dict exists (by default in write_empty_rnb)
    assert_eq!(
        f.directory.entries.iter().filter(|e| e.segment_type == rnb_format::SegmentType::StringDict.as_u32()).count(),
        1
    );
    assert!(f.string_dict.is_some());

    let _ = std::fs::remove_file(&p);
}