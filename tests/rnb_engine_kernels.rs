#[test]
fn engine_objects_by_type_integration() {
    use rnb_format::{Manifest, ObjectRecord, ObjectTable};

    // Build a small object table with two distinct type_sids.
    let manifest = Manifest::minimal();

    let mut ot = ObjectTable::empty();
    ot.push(ObjectRecord { type_sid: 1, name_sid: 10, flags: 0 });
    ot.push(ObjectRecord { type_sid: 2, name_sid: 20, flags: 0 });
    ot.push(ObjectRecord { type_sid: 1, name_sid: 30, flags: 0 });

    let mut p = std::env::temp_dir();
    p.push(format!("rnb_engine_kernels_{}.rnb", std::process::id()));

    rnb_format::write_minimal_rnb(&p, &manifest, None, Some(&ot)).unwrap();

    let art = rnb_engine::open(&p).unwrap();

    // objects_by_type should return exactly the objects whose type_sid matches.
    let type1 = art.objects_by_type(1).unwrap();
    assert_eq!(type1.len(), 2);
    assert!(type1.iter().all(|o| o.type_sid == 1));

    let type2 = art.objects_by_type(2).unwrap();
    assert_eq!(type2.len(), 1);
    assert_eq!(type2[0].type_sid, 2);

    let none = art.objects_by_type(999).unwrap();
    assert!(none.is_empty());

    let _ = std::fs::remove_file(&p);
}

