#[test]
fn attribute_table_roundtrip() {
    let table = rnb_format::AttributeTable {
        attributes: vec![
            rnb_format::AttributeRecord { object_id: 0, key_sid: 1, value_sid: 10, flags: 0 },
            rnb_format::AttributeRecord { object_id: 1, key_sid: 2, value_sid: 20, flags: 0 },
            rnb_format::AttributeRecord { object_id: 1, key_sid: 3, value_sid: 30, flags: 1 },
        ],
    };

    let mut bytes = Vec::new();
    table.write_to(&mut bytes).unwrap();

    let table2 = rnb_format::AttributeTable::read_from(&bytes[..]).unwrap();

    assert_eq!(table, table2);
    assert_eq!(table2.len(), 3);
    assert!(!table2.is_empty());
}

