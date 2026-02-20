#[test]
fn relation_table_roundtrip() {
    let table = rnb_format::RelationTable {
        relations: vec![
            rnb_format::RelationRecord { src_id: 0, dst_id: 1, rel_type_sid: 100, flags: 0 },
            rnb_format::RelationRecord { src_id: 1, dst_id: 2, rel_type_sid: 200, flags: 0 },
        ],
    };

    let mut bytes = Vec::new();
    table.write_to(&mut bytes).unwrap();

    let table2 = rnb_format::RelationTable::read_from(&bytes[..]).unwrap();

    assert_eq!(table, table2);
    assert_eq!(table2.len(), 2);
    assert!(!table2.is_empty());
}

