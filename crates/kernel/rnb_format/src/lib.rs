// crates/kernel/rnb_format/src/lib.rs

mod header;
mod directory;
mod segment;
mod manifest;
mod string_dict;
mod object_table;
mod attribute_table;
mod relation_table;
mod numeric_matrix;

pub use header::{RnbHeader, RNB_MAGIC, RNB_VERSION_MAJOR, RNB_VERSION_MINOR};
pub use directory::{RnbDirectory, RnbDirEntry};
pub use segment::{SegmentType, QueryKernel};
pub use manifest::{Manifest, checksum64_fnv1a};
pub use string_dict::StringDict;
pub use object_table::{ObjectTable, ObjectRecord};
pub use attribute_table::{AttributeTable, AttributeRecord};
pub use relation_table::{RelationTable, RelationRecord};
pub use numeric_matrix::{NumericMatrix, NumericType};

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RnbFile {
    pub header: RnbHeader,
    pub directory: RnbDirectory,
    pub manifest: Manifest,
    pub string_dict: Option<StringDict>,
}

pub fn write_empty_rnb(path: impl AsRef<Path>) -> std::io::Result<()> {
    // Minimal valid artifact now includes manifest + (for commit 5) an empty dict by default.
    write_minimal_rnb(path, &Manifest::minimal(), Some(&StringDict::empty()))
}

pub fn write_minimal_rnb(
    path: impl AsRef<Path>,
    manifest: &Manifest,
    string_dict: Option<&StringDict>,
) -> std::io::Result<()> {
    let mut f = File::create(path)?;

    let mut header = RnbHeader::new();
    header.write_to(&mut f)?;

    // --- Manifest segment ---
    let mut manifest_bytes: Vec<u8> = Vec::new();
    manifest.write_to(&mut manifest_bytes)?;
    let manifest_checksum = checksum64_fnv1a(&manifest_bytes);
    let manifest_offset = f.stream_position()?;
    f.write_all(&manifest_bytes)?;
    let manifest_len = manifest_bytes.len() as u64;

    // --- StringDict segment (optional) ---
    let mut dict_entry: Option<RnbDirEntry> = None;
    if let Some(sd) = string_dict {
        let dict_bytes = sd.to_bytes()?;
        let dict_checksum = checksum64_fnv1a(&dict_bytes);
        let dict_offset = f.stream_position()?;
        f.write_all(&dict_bytes)?;
        let dict_len = dict_bytes.len() as u64;

        dict_entry = Some(RnbDirEntry {
            segment_id: 2,
            segment_type: SegmentType::StringDict.as_u32(),
            offset: dict_offset,
            length: dict_len,
            checksum64: dict_checksum,
        });
    }

    // --- Directory at end ---
    let dir_offset = f.stream_position()?;
    let mut entries = Vec::new();
    entries.push(RnbDirEntry {
        segment_id: 1,
        segment_type: SegmentType::Manifest.as_u32(),
        offset: manifest_offset,
        length: manifest_len,
        checksum64: manifest_checksum,
    });
    if let Some(e) = dict_entry {
        entries.push(e);
    }

    let dir = RnbDirectory { entries };
    dir.write_to(&mut f)?;
    let dir_end = f.stream_position()?;

    header.dir_offset = dir_offset;
    header.dir_len = dir_end - dir_offset;

    f.seek(SeekFrom::Start(0))?;
    header.write_to(&mut f)?;

    Ok(())
}

pub fn open_rnb(path: impl AsRef<Path>) -> std::io::Result<RnbFile> {
    let mut f = File::open(path)?;
    let header = RnbHeader::read_from(&mut f)?;

    f.seek(SeekFrom::Start(header.dir_offset))?;
    let directory = RnbDirectory::read_from(&mut f, header.dir_len)?;

    // --- Read manifest ---
    let manifest_entry = directory
        .entries
        .iter()
        .find(|e| e.segment_type == SegmentType::Manifest.as_u32())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing manifest"))?;

    let manifest_bytes = read_segment_bytes(&mut f, manifest_entry)?;
    let got = checksum64_fnv1a(&manifest_bytes);
    if got != manifest_entry.checksum64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "manifest checksum mismatch",
        ));
    }
    let manifest = Manifest::read_from(&manifest_bytes[..])?;

    // Ensure that all manifest-declared required segments are actually present
    // in the directory. This ties the semantic contract (manifest) to the
    // physical layout (directory entries).
    for req in &manifest.required_segments {
        let st = req.as_u32();
        let found = directory.entries.iter().any(|e| e.segment_type == st);
        if !found {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing required segment declared in manifest",
            ));
        }
    }

    // --- Read optional StringDict ---
    let dict_entry = directory
        .entries
        .iter()
        .find(|e| e.segment_type == SegmentType::StringDict.as_u32());

    let string_dict = if let Some(e) = dict_entry {
        let bytes = read_segment_bytes(&mut f, e)?;
        let got = checksum64_fnv1a(&bytes);
        if got != e.checksum64 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "string dict checksum mismatch",
            ));
        }
        Some(StringDict::from_bytes(&bytes[..])?)
    } else {
        None
    };

    Ok(RnbFile {
        header,
        directory,
        manifest,
        string_dict,
    })
}

fn read_segment_bytes(f: &mut File, e: &RnbDirEntry) -> std::io::Result<Vec<u8>> {
    f.seek(SeekFrom::Start(e.offset))?;
    let mut buf = vec![0u8; e.length as usize];
    f.read_exact(&mut buf)?;
    Ok(buf)
}
