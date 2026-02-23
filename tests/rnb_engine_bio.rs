#[test]
fn bio_view_cells_and_genes_by_label() {
    use rnb_format::{Manifest, ObjectRecord, ObjectTable, StringDict};

    // StringDict with canonical type labels in known positions.
    let dict = StringDict::new(vec![
        "cell".to_string(), // type_sid = 0
        "gene".to_string(), // type_sid = 1
    ]);

    // Object table with a mix of "cell" and "gene" objects.
    let mut ot = ObjectTable::empty();
    ot.push(ObjectRecord { type_sid: 0, name_sid: 10, flags: 0 });
    ot.push(ObjectRecord { type_sid: 1, name_sid: 20, flags: 0 });
    ot.push(ObjectRecord { type_sid: 0, name_sid: 30, flags: 0 });

    let manifest = Manifest::minimal();

    let mut p = std::env::temp_dir();
    p.push(format!("rnb_engine_bio_cells_genes_{}.rnb", std::process::id()));

    // Write manifest + dict + object table into a single artifact.
    rnb_format::write_minimal_rnb(&p, &manifest, Some(&dict), Some(&ot)).unwrap();

    let art = rnb_engine::open(&p).unwrap();
    let bio = rnb_engine::BioView::from_artifact(&art).expect("BioView available");

    let cells = bio.cells().unwrap();
    assert_eq!(cells.len(), 2);
    assert!(cells.iter().all(|o| o.type_sid == 0));

    let genes = bio.genes().unwrap();
    assert_eq!(genes.len(), 1);
    assert_eq!(genes[0].type_sid, 1);

    let _ = std::fs::remove_file(&p);
}

#[test]
fn bio_view_missing_labels_fail_cleanly() {
    use rnb_format::{Manifest, ObjectRecord, ObjectTable, StringDict};

    // Dict without "gene" label; only "cell" is present.
    let dict = StringDict::new(vec!["cell".to_string()]);

    let mut ot = ObjectTable::empty();
    ot.push(ObjectRecord { type_sid: 0, name_sid: 10, flags: 0 });

    let manifest = Manifest::minimal();

    let mut p = std::env::temp_dir();
    p.push(format!("rnb_engine_bio_missing_{}.rnb", std::process::id()));

    rnb_format::write_minimal_rnb(&p, &manifest, Some(&dict), Some(&ot)).unwrap();

    let art = rnb_engine::open(&p).unwrap();
    let bio = rnb_engine::BioView::from_artifact(&art).expect("BioView available");

    // "cell" should work.
    let cells = bio.cells().unwrap();
    assert_eq!(cells.len(), 1);

    // "gene" should produce a clean InvalidInput error.
    let err = bio.genes().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);

    let _ = std::fs::remove_file(&p);
}

