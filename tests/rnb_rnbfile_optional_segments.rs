use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

use rnb_format::{
    AttributeRecord,
    AttributeTable,
    Manifest,
    NumericMatrix,
    QueryKernel,
    RelationRecord,
    RelationTable,
    RnbDirectory,
    RnbDirEntry,
    RnbHeader,
    SegmentType,
    StringDict,
    checksum64_fnv1a,
};

fn temp_path(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("rnbfile_optional_{}_{}.rnb", tag, std::process::id()));
    p
}

#[test]
fn rnbfile_populates_string_dict_segment() {
    let path = temp_path("string_dict");

    // Use the high-level helper which writes a manifest + empty StringDict.
    rnb_format::write_empty_rnb(&path).unwrap();

    let f = rnb_format::open_rnb(&path).unwrap();
    let dict = f.string_dict.as_ref().expect("string dict present");

    assert!(!dict.is_empty());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn rnbfile_populates_object_table_segment() {
    use rnb_format::{ObjectRecord, ObjectTable};

    let path = temp_path("object_table");

    let manifest = Manifest::minimal();

    let mut ot = ObjectTable::empty();
    ot.push(ObjectRecord { type_sid: 1, name_sid: 10, flags: 0 });
    ot.push(ObjectRecord { type_sid: 2, name_sid: 20, flags: 1 });

    // Manifest + object table, using the existing helper.
    rnb_format::write_minimal_rnb(&path, &manifest, None, Some(&ot)).unwrap();

    let f = rnb_format::open_rnb(&path).unwrap();
    let table = f.object_table.as_ref().expect("object table present");

    assert_eq!(table.len(), 2);
    assert_eq!(table.objects[0].type_sid, 1);
    assert_eq!(table.objects[1].type_sid, 2);

    let _ = std::fs::remove_file(&path);
}

fn write_rnb_with_single_segment(
    path: &PathBuf,
    segment_type: SegmentType,
    segment_bytes: &[u8],
) -> std::io::Result<()> {
    let mut f = File::create(path)?;

    let mut header = RnbHeader::new();
    header.write_to(&mut f)?;

    // Manifest first.
    let manifest = Manifest::minimal();
    let mut manifest_bytes = Vec::new();
    manifest.write_to(&mut manifest_bytes)?;
    let manifest_checksum = checksum64_fnv1a(&manifest_bytes);
    let manifest_offset = f.stream_position()?;
    f.write_all(&manifest_bytes)?;
    let manifest_len = manifest_bytes.len() as u64;

    // Then the single payload segment.
    let segment_checksum = checksum64_fnv1a(segment_bytes);
    let segment_offset = f.stream_position()?;
    f.write_all(segment_bytes)?;
    let segment_len = segment_bytes.len() as u64;

    // Directory.
    let dir_offset = f.stream_position()?;
    let entries = vec![
        RnbDirEntry {
            segment_id: 1,
            segment_type: SegmentType::Manifest.as_u32(),
            offset: manifest_offset,
            length: manifest_len,
            checksum64: manifest_checksum,
        },
        RnbDirEntry {
            segment_id: 2,
            segment_type: segment_type.as_u32(),
            offset: segment_offset,
            length: segment_len,
            checksum64: segment_checksum,
        },
    ];
    let dir = RnbDirectory { entries };
    dir.write_to(&mut f)?;
    let dir_end = f.stream_position()?;

    header.dir_offset = dir_offset;
    header.dir_len = dir_end - dir_offset;

    f.seek(SeekFrom::Start(0))?;
    header.write_to(&mut f)?;

    Ok(())
}

#[test]
fn rnbfile_populates_attribute_table_segment() {
    let path = temp_path("attribute_table");

    let table = AttributeTable {
        attributes: vec![
            AttributeRecord { object_id: 0, key_sid: 1, value_sid: 10, flags: 0 },
            AttributeRecord { object_id: 1, key_sid: 2, value_sid: 20, flags: 0 },
        ],
    };

    let mut bytes = Vec::new();
    table.write_to(&mut bytes).unwrap();

    write_rnb_with_single_segment(&path, SegmentType::AttributeTable, &bytes).unwrap();

    let f = rnb_format::open_rnb(&path).unwrap();
    let table2 = f.attribute_table.as_ref().expect("attribute table present");

    assert_eq!(table2.len(), 2);
    assert!(table2.attributes.iter().any(|a| a.key_sid == 1 && a.value_sid == 10));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn rnbfile_populates_relation_table_segment() {
    let path = temp_path("relation_table");

    let table = RelationTable {
        relations: vec![
            RelationRecord { src_id: 0, dst_id: 1, rel_type_sid: 100, flags: 0 },
            RelationRecord { src_id: 1, dst_id: 2, rel_type_sid: 200, flags: 0 },
        ],
    };

    let mut bytes = Vec::new();
    table.write_to(&mut bytes).unwrap();

    write_rnb_with_single_segment(&path, SegmentType::RelationTable, &bytes).unwrap();

    let f = rnb_format::open_rnb(&path).unwrap();
    let table2 = f.relation_table.as_ref().expect("relation table present");

    assert_eq!(table2.len(), 2);
    assert!(table2.relations.iter().any(|r| r.rel_type_sid == 100));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn rnbfile_populates_numeric_matrix_segment() {
    let path = temp_path("numeric_matrix");

    let values = vec![
        1.0f32, 2.0, 3.0,
        4.0, 5.0, 6.0,
    ];
    let matrix = NumericMatrix::new(2, 3, values.clone()).unwrap();

    let mut bytes = Vec::new();
    matrix.write_to(&mut bytes).unwrap();

    write_rnb_with_single_segment(&path, SegmentType::NumericMatrix, &bytes).unwrap();

    let f = rnb_format::open_rnb(&path).unwrap();
    let m2 = f.numeric_matrix.as_ref().expect("numeric matrix present");

    assert_eq!(m2.rows, 2);
    assert_eq!(m2.cols, 3);
    assert_eq!(m2.values, values);

    let _ = std::fs::remove_file(&path);
}

