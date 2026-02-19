// crates/kernel/rnb_format/src/directory.rs

use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RnbDirEntry {
    pub segment_id: u32,
    pub segment_type: u32,
    pub offset: u64,
    pub length: u64,
    pub checksum64: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RnbDirectory {
    pub entries: Vec<RnbDirEntry>,
}

impl RnbDirectory {
    pub fn empty() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn entry_count(&self) -> u32 {
        self.entries.len() as u32
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        let count = self.entry_count();
        w.write_all(&count.to_le_bytes())?;
        w.write_all(&0u32.to_le_bytes())?; // reserved

        for e in &self.entries {
            w.write_all(&e.segment_id.to_le_bytes())?;
            w.write_all(&e.segment_type.to_le_bytes())?;
            w.write_all(&e.offset.to_le_bytes())?;
            w.write_all(&e.length.to_le_bytes())?;
            w.write_all(&e.checksum64.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R, dir_len: u64) -> std::io::Result<Self> {
        // Directory header: count (u32) + reserved (u32) = 8 bytes
        if dir_len < 8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "directory too small",
            ));
        }

        let mut buf4 = [0u8; 4];
        r.read_exact(&mut buf4)?;
        let count = u32::from_le_bytes(buf4) as usize;

        r.read_exact(&mut buf4)?;
        let reserved = u32::from_le_bytes(buf4);
        if reserved != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "directory reserved must be 0",
            ));
        }

        // Each entry is 32 bytes
        let expected = 8u64 + (count as u64) * 32u64;
        if dir_len != expected {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "directory length mismatch",
            ));
        }

        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            r.read_exact(&mut buf4)?;
            let segment_id = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let segment_type = u32::from_le_bytes(buf4);

            let mut buf8 = [0u8; 8];
            r.read_exact(&mut buf8)?;
            let offset = u64::from_le_bytes(buf8);

            r.read_exact(&mut buf8)?;
            let length = u64::from_le_bytes(buf8);

            r.read_exact(&mut buf8)?;
            let checksum64 = u64::from_le_bytes(buf8);

            entries.push(RnbDirEntry {
                segment_id,
                segment_type,
                offset,
                length,
                checksum64,
            });
        }

        Ok(Self { entries })
    }
}