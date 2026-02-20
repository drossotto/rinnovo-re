#[test]
fn object_table_roundtrip() {
    let table = rnb_format::ObjectTable {
        objects: vec![
            rnb_format::ObjectRecord { type_sid: 1, name_sid: 10, flags: 0 },
            rnb_format::ObjectRecord { type_sid: 2, name_sid: 20, flags: 1 },
            rnb_format::ObjectRecord { type_sid: 3, name_sid: 30, flags: 0 },
        ],
    };

    let mut bytes = Vec::new();
    table.write_to(&mut bytes).unwrap();

    let table2 = rnb_format::ObjectTable::read_from(&bytes[..]).unwrap();

    assert_eq!(table, table2);
    assert_eq!(table2.len(), 3);
    assert!(!table2.is_empty());
}

