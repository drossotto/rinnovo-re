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