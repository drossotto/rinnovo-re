use std::path::Path;

pub use rnb_format::{
    AttributeRecord,
    AttributeTable,
    Manifest,
    NumericMatrix,
    NumericType,
    ObjectRecord,
    ObjectTable,
    QueryKernel,
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

/// Open an RNB artifact and parse the header, directory, manifest,
/// and any required invariants enforced at the container level.
pub fn open(path: impl AsRef<Path>) -> std::io::Result<RnbFile> {
    rnb_format::open_rnb(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let mut p = std::env::temp_dir();
        p.push(format!("rnb_engine_roundtrip_{}.rnb", std::process::id()));

        write_empty(&p).unwrap();
        let f = open(&p).unwrap();

        assert_eq!(&f.header.magic, b"RNB\0");
        assert!(!f.directory.entries.is_empty());

        let _ = std::fs::remove_file(&p);
    }
}
