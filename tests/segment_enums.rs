#[test]
fn segment_type_roundtrip() {
    use rnb_format::SegmentType;

    let all = [
        (1u32, SegmentType::Manifest),
        (2, SegmentType::StringDict),
        (3, SegmentType::ObjectTable),
        (4, SegmentType::AttributeTable),
        (5, SegmentType::RelationTable),
        (6, SegmentType::NumericMatrix),
    ];

    for (raw, variant) in all {
        assert_eq!(SegmentType::from_u32(raw), Some(variant));
        assert_eq!(variant.as_u32(), raw);
    }

    // Unknown codes should return None.
    assert_eq!(SegmentType::from_u32(0), None);
    assert_eq!(SegmentType::from_u32(999), None);
}

#[test]
fn query_kernel_roundtrip() {
    use rnb_format::QueryKernel;

    let all = [
        (1u32, QueryKernel::GetObjectById),
        (2, QueryKernel::ObjectsByType),
    ];

    for (raw, variant) in all {
        assert_eq!(QueryKernel::from_u32(raw), Some(variant));
        assert_eq!(variant.as_u32(), raw);
    }

    assert_eq!(QueryKernel::from_u32(0), None);
    assert_eq!(QueryKernel::from_u32(999), None);
}

