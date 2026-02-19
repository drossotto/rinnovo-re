// crates/kernel/rnb_format/src/lib.rs

mod header;
mod directory;
mod segment;
mod manifest;

pub use header::{
    RnbHeader, 
    RNB_MAGIC, 
    RNB_VERSION_MAJOR, 
    RNB_VERSION_MINOR
};
pub use directory::{
    RnbDirectory, 
    RnbDirEntry
};
pub use segment::{
    SegmentType, 
    QueryKernel
};
pub use manifest::{
    Manifest, 
    checksum64_fnv1a
};

use std::fs::File;
use std::io::{
    Read, 
    Seek, 
    SeekFrom, 
    Write
};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RnbFile {
    pub header: RnbHeader,
    pub directory: RnbDirectory,
    pub manifest: Manifest,
}

pub fn write_empty_rnb(path: impl AsRef<Path>) -> std::io::Result<()> {
    write_minimal_rnb(path, &Manifest::minimal())
}

pub fn write_minimal_rnb(path: impl AsRef<Path>, manifest: &Manifest) -> std::io::Result<()> {
    let mut f = File::create(path)?;

    // TODO: Patch dir_offset/dir_len
    let mut header = RnbHeader::new();
    header.write_to(&mut f)?;

    // Write manifest bytes to a buffer so we can checksum + know length
    let mut manifest_bytes: Vec<u8> = Vec::new();
    manifest.write_to(&mut manifest_bytes)?;
    let manifest_checksum = checksum64_fnv1a(&manifest_bytes);

    // Write manifest segment to file
    let manifest_offset = f.stream_position()?;
    f.write_all(&manifest_bytes)?;
    let manifest_len = manifest_bytes.len() as u64;

    // Write directory at end with one entry (manifest)
    let dir_offset = f.stream_position()?;
    let dir = RnbDirectory {
        entries: vec![RnbDirEntry {
            segment_id: 1,
            segment_type: SegmentType::Manifest.as_u32(),
            offset: manifest_offset,
            length: manifest_len,
            checksum64: manifest_checksum,
        }],
    };
    dir.write_to(&mut f)?;
    let dir_end = f.stream_position()?;

    // Patch header with directory location
    header.dir_offset = dir_offset;
    header.dir_len = dir_end - dir_offset;

    // Rewrite header at file start
    f.seek(SeekFrom::Start(0))?;
    header.write_to(&mut f)?;

    Ok(())
}

/// Opens an .rnb and returns the parsed header, directory, and manifest.
/// Open-time cost is bounded: header + directory + manifest.
pub fn open_rnb(path: impl AsRef<Path>) -> std::io::Result<RnbFile> {
    let mut f = File::open(path)?;

    let header = RnbHeader::read_from(&mut f)?;

    // Read directory
    f.seek(SeekFrom::Start(header.dir_offset))?;
    let directory = RnbDirectory::read_from(&mut f, header.dir_len)?;

    // Find manifest entry
    let manifest_entry = directory
        .entries
        .iter()
        .find(|e| e.segment_type == SegmentType::Manifest.as_u32())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing manifest segment"))?;

    // Read manifest bytes
    f.seek(SeekFrom::Start(manifest_entry.offset))?;
    let mut manifest_bytes = vec![0u8; manifest_entry.length as usize];
    f.read_exact(&mut manifest_bytes)?;

    // Verify checksum
    let got = checksum64_fnv1a(&manifest_bytes);
    if got != manifest_entry.checksum64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "manifest checksum mismatch",
        ));
    }

    // Parse manifest
    let manifest = Manifest::read_from(&manifest_bytes[..])?;

    Ok(RnbFile {
        header,
        directory,
        manifest,
    })
}