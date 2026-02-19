// crates/kernel/rnb_format/src/lib.rs
mod header;
mod directory;

pub use header::{RnbHeader, RNB_MAGIC, RNB_VERSION_MAJOR, RNB_VERSION_MINOR};
pub use directory::{RnbDirectory, RnbDirEntry};

use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::Path;

pub fn write_empty_rnb(path: impl AsRef<Path>) -> std::io::Result<()> {
    let mut f = File::create(path)?;

    // Write placeholder header
    let mut header = RnbHeader::new();
    header.write_to(&mut f)?;

    // Write directory immediately after header
    let dir_offset = f.stream_position()?;
    let dir = RnbDirectory::empty();
    dir.write_to(&mut f)?;
    let dir_end = f.stream_position()?;

    // Patch header and rewrite at start
    header.dir_offset = dir_offset;
    header.dir_len = dir_end - dir_offset;

    f.seek(SeekFrom::Start(0))?;
    header.write_to(&mut f)?;

    Ok(())
}

pub fn open_rnb(path: impl AsRef<Path>) -> std::io::Result<(RnbHeader, RnbDirectory)> {
    let mut f = File::open(path)?;
    let header = RnbHeader::read_from(&mut f)?;

    f.seek(SeekFrom::Start(header.dir_offset))?;
    let dir = RnbDirectory::read_from(&mut f, header.dir_len)?;

    Ok((header, dir))
}