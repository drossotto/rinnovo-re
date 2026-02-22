use std::path::Path;

pub mod artifact;
pub mod object;

pub use artifact::Artifact;
pub use object::Object;

pub use rnb_format::{
    AttributeRecord,
    AttributeTable,
    Manifest,
    NumericMatrix,
    NumericType,
    ObjectRecord,
    ObjectTable,
    QueryKernel,
    RelationRecord,
    RelationTable,
    RnbDirectory,
    RnbDirEntry,
    RnbFile,
    RnbHeader,
    SegmentType,
    StringDict,
};

/// Write a minimal RNB artifact to the given path.
///
/// This currently writes a single manifest segment and an empty string
/// dictionary by default. The exact contents may evolve, but the
/// resulting file is always a valid RNB artifact.
pub fn write_empty(path: impl AsRef<Path>) -> std::io::Result<()> {
    rnb_format::write_empty_rnb(path)
}

/// Open an RNB artifact and return a high-level `Artifact` wrapper.
pub fn open(path: impl AsRef<Path>) -> std::io::Result<Artifact> {
    Artifact::open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let mut p = std::env::temp_dir();
        p.push(format!("rnb_engine_roundtrip_{}.rnb", std::process::id()));

        write_empty(&p).unwrap();
        let art = open(&p).unwrap();

        assert_eq!(&art.header().magic, b"RNB\0");
        assert!(!art.directory().entries.is_empty());
        assert!(art.manifest().required_segments.contains(&SegmentType::Manifest));

        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn artifact_exposes_object_table() {
        use rnb_format::{Manifest, ObjectRecord, ObjectTable};

        let manifest = Manifest::minimal();

        // Build a small object table.
        let mut ot = ObjectTable::empty();
        ot.push(ObjectRecord { type_sid: 1, name_sid: 10, flags: 0 });
        ot.push(ObjectRecord { type_sid: 2, name_sid: 20, flags: 1 });

        let mut p = std::env::temp_dir();
        p.push(format!("rnb_engine_objects_{}.rnb", std::process::id()));

        // Write an artifact with manifest + object table (no string dict needed yet).
        rnb_format::write_minimal_rnb(&p, &manifest, None, Some(&ot)).unwrap();

        let art = open(&p).unwrap();

        // The object table should be visible through the Artifact wrapper.
        assert_eq!(art.object_count(), Some(2));

        let o0 = art.get_object(0).expect("object 0");
        assert_eq!(o0.id, 0);
        assert_eq!(o0.type_sid, 1);
        assert_eq!(o0.name_sid, 10);
        assert_eq!(o0.flags, 0);

        let o1 = art.get_object(1).expect("object 1");
        assert_eq!(o1.id, 1);
        assert_eq!(o1.type_sid, 2);
        assert_eq!(o1.name_sid, 20);
        assert_eq!(o1.flags, 1);

        // Out of range IDs should return None.
        assert!(art.get_object(2).is_none());

        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn artifact_execute_get_object_by_id_kernel() {
        use rnb_format::{Manifest, ObjectRecord, ObjectTable, QueryKernel};

        let manifest = Manifest::minimal();

        let mut ot = ObjectTable::empty();
        ot.push(ObjectRecord { type_sid: 1, name_sid: 10, flags: 0 });

        let mut p = std::env::temp_dir();
        p.push(format!("rnb_engine_execute_{}.rnb", std::process::id()));

        rnb_format::write_minimal_rnb(&p, &manifest, None, Some(&ot)).unwrap();
        let art = open(&p).unwrap();

        // Minimal manifest should advertise GetObjectById support.
        assert!(art.manifest().supported_kernels.contains(&QueryKernel::GetObjectById));

        let result = art.execute(QueryKernel::GetObjectById, 0).unwrap();
        assert_eq!(result.len(), 1);
        let obj = &result[0];
        assert_eq!(obj.id, 0);
        assert_eq!(obj.type_sid, 1);
        assert_eq!(obj.name_sid, 10);

        let missing = art.execute(QueryKernel::GetObjectById, 1).unwrap();
        assert!(missing.is_empty());

        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn artifact_execute_objects_by_type_kernel() {
        use rnb_format::{Manifest, ObjectRecord, ObjectTable, QueryKernel};

        let manifest = Manifest::minimal();

        let mut ot = ObjectTable::empty();
        ot.push(ObjectRecord { type_sid: 1, name_sid: 10, flags: 0 });
        ot.push(ObjectRecord { type_sid: 2, name_sid: 20, flags: 0 });
        ot.push(ObjectRecord { type_sid: 1, name_sid: 30, flags: 0 });

        let mut p = std::env::temp_dir();
        p.push(format!("rnb_engine_execute_type_{}.rnb", std::process::id()));

        rnb_format::write_minimal_rnb(&p, &manifest, None, Some(&ot)).unwrap();
        let art = open(&p).unwrap();

        assert!(art.manifest().supported_kernels.contains(&QueryKernel::ObjectsByType));

        // Use the generic execute API.
        let objs = art.execute(QueryKernel::ObjectsByType, 1).unwrap();
        assert_eq!(objs.len(), 2);
        assert_eq!(objs[0].type_sid, 1);
        assert_eq!(objs[1].type_sid, 1);

        // And the convenience wrapper.
        let objs2 = art.objects_by_type(2).unwrap();
        assert_eq!(objs2.len(), 1);
        assert_eq!(objs2[0].type_sid, 2);

        let _ = std::fs::remove_file(&p);
    }
}
