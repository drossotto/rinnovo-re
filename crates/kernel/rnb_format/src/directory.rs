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
    pub entry_count: u32,
    pub entries: Vec<RnbDirEntry>,
}

impl RnbDirectory {
    pub fn empty() -> Self {
        Self {
            entry_count: 0,
            entries: Vec::new(),
        }
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&self.entry_count.to_le_bytes())?;
        w.write_all(&0u32.to_le_bytes())?; // reserved
        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R, _dir_len: u64) -> std::io::Result<Self> {
        let mut buf4 = [0u8; 4];
        r.read_exact(&mut buf4)?;
        let entry_count = u32::from_le_bytes(buf4);

        r.read_exact(&mut buf4)?; // reserved (ignored for now)

        Ok(Self {
            entry_count,
            entries: Vec::new(),
        })
    }
}