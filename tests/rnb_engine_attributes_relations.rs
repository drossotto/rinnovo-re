#[test]
fn engine_attributes_for_object_and_relations_roundtrip() {
    use rnb_format::{
        AttributeRecord,
        AttributeTable,
        Manifest,
        ObjectRecord,
        ObjectTable,
        RelationRecord,
        RelationTable,
    };

    // Minimal manifest and object table for three objects.
    let manifest = Manifest::minimal();

    let mut ot = ObjectTable::empty();
    ot.push(ObjectRecord { type_sid: 1, name_sid: 10, flags: 0 }); // id 0
    ot.push(ObjectRecord { type_sid: 1, name_sid: 20, flags: 0 }); // id 1
    ot.push(ObjectRecord { type_sid: 2, name_sid: 30, flags: 0 }); // id 2

    // Attribute table: two attributes for object 1, one for object 2.
    let mut at = AttributeTable::empty();
    at.push(AttributeRecord { object_id: 1, key_sid: 100, value_sid: 1000, flags: 0 });
    at.push(AttributeRecord { object_id: 1, key_sid: 101, value_sid: 1001, flags: 0 });
    at.push(AttributeRecord { object_id: 2, key_sid: 200, value_sid: 2000, flags: 0 });

    // Relation table: 0 -> 1 (type 10), 1 -> 2 (type 10), 2 -> 0 (type 20).
    let mut rt = RelationTable::empty();
    rt.push(RelationRecord { src_id: 0, dst_id: 1, rel_type_sid: 10, flags: 0 });
    rt.push(RelationRecord { src_id: 1, dst_id: 2, rel_type_sid: 10, flags: 0 });
    rt.push(RelationRecord { src_id: 2, dst_id: 0, rel_type_sid: 20, flags: 0 });

    // Write an artifact containing manifest + object + attribute + relation tables.
    let mut p = std::env::temp_dir();
    p.push(format!("rnb_engine_attrs_rels_{}.rnb", std::process::id()));

    rnb_format::write_minimal_rnb_with_optional_segments(
        &p,
        &manifest,
        None,
        Some(&ot),
        Some(&at),
        Some(&rt),
    )
    .unwrap();

    let art = rnb_engine::open(&p).unwrap();

    // Attribute helpers: object 1 has exactly two attributes.
    let attrs_obj1: Vec<_> = art
        .attributes_for_object(1)
        .expect("attribute table present")
        .collect();
    assert_eq!(attrs_obj1.len(), 2);
    assert!(attrs_obj1.iter().any(|a| a.key_sid == 100 && a.value_sid == 1000));
    assert!(attrs_obj1.iter().any(|a| a.key_sid == 101 && a.value_sid == 1001));

    // Object with no attributes should yield an empty iterator.
    let attrs_obj0: Vec<_> = art
        .attributes_for_object(0)
        .expect("attribute table present")
        .collect();
    assert!(attrs_obj0.is_empty());

    // Relation helpers: from src=1 with type 10 returns the 1 -> 2 edge.
    let rels_from1_type10: Vec<_> = art
        .relations_from(1, Some(10))
        .expect("relation table present")
        .collect();
    assert_eq!(rels_from1_type10.len(), 1);
    assert_eq!(rels_from1_type10[0].src_id, 1);
    assert_eq!(rels_from1_type10[0].dst_id, 2);
    assert_eq!(rels_from1_type10[0].rel_type_sid, 10);

    // All relations from src=0 regardless of type.
    let rels_from0: Vec<_> = art
        .relations_from(0, None)
        .expect("relation table present")
        .collect();
    assert_eq!(rels_from0.len(), 1);
    assert_eq!(rels_from0[0].dst_id, 1);

    // Relations targeting dst=0, filtered by type 20.
    let rels_to0_type20: Vec<_> = art
        .relations_to(0, Some(20))
        .expect("relation table present")
        .collect();
    assert_eq!(rels_to0_type20.len(), 1);
    assert_eq!(rels_to0_type20[0].src_id, 2);
    assert_eq!(rels_to0_type20[0].rel_type_sid, 20);

    // Non-existent type should return an empty iterator.
    let rels_to0_type999: Vec<_> = art
        .relations_to(0, Some(999))
        .expect("relation table present")
        .collect();
    assert!(rels_to0_type999.is_empty());

    let _ = std::fs::remove_file(&p);
}

