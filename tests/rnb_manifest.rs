// tests/rnb_manifest.rs

#[test]
fn manifest_serialization_roundtrip() {
    let m = rnb_format::Manifest::minimal();

    let mut bytes = Vec::new();
    m.write_to(&mut bytes).unwrap();

    let m2 = rnb_format::Manifest::read_from(&bytes[..]).unwrap();

    assert_eq!(m, m2);
    assert_eq!(m2.required_segments, vec![rnb_format::SegmentType::Manifest]);
    assert!(m2.max_chunk_bytes >= 1024);
}

#[test]
fn open_rnb_enforces_required_segments() {
    use rnb_format::{Manifest, SegmentType, StringDict};

    // Start from the minimal manifest and declare that StringDict is required.
    let mut m = Manifest::minimal();
    m.required_segments.push(SegmentType::StringDict);

    let mut p = std::env::temp_dir();
    p.push(format!(
        "rnb_missing_required_{}.rnb",
        std::process::id()
    ));

    // Write a file that has only the manifest segment (no StringDict segment),
    // but whose manifest declares StringDict as required.
    //
    // We pass `None` for the string_dict argument so that only the manifest
    // segment is written to the directory.
    rnb_format::write_minimal_rnb(&p, &m, None).unwrap();

    // open_rnb must now fail because the required StringDict segment is missing.
    let err = rnb_format::open_rnb(&p).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);

    let _ = std::fs::remove_file(&p);
}
