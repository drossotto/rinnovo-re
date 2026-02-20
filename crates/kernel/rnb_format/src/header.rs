// crates/kernel/rnb_format/src/header.rs

use std::io::{Read, Write};

pub const RNB_MAGIC: [u8; 4] = *b"RNB\0";
pub const RNB_VERSION_MAJOR: u16 = 0;
pub const RNB_VERSION_MINOR: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RnbHeader {
    pub magic: [u8; 4],
    pub version_major: u16,
    pub version_minor: u16,
    pub dir_offset: u64,
    pub dir_len: u64,
}

impl RnbHeader {
    pub fn new() -> Self {
        Self {
            magic: RNB_MAGIC,
            version_major: RNB_VERSION_MAJOR,
            version_minor: RNB_VERSION_MINOR,
            dir_offset: 0,
            dir_len: 0,
        }
    }

    pub fn validate(&self) -> std::io::Result<()> {
        if self.magic != RNB_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid RNB magic",
            ));
        }
        Ok(())
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&self.magic)?;
        w.write_all(&self.version_major.to_le_bytes())?;
        w.write_all(&self.version_minor.to_le_bytes())?;
        w.write_all(&self.dir_offset.to_le_bytes())?;
        w.write_all(&self.dir_len.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let version_major = u16::from_le_bytes(buf2);

        r.read_exact(&mut buf2)?;
        let version_minor = u16::from_le_bytes(buf2);

        let mut buf8 = [0u8; 8];
        r.read_exact(&mut buf8)?;
        let dir_offset = u64::from_le_bytes(buf8);

        r.read_exact(&mut buf8)?;
        let dir_len = u64::from_le_bytes(buf8);

        let header = Self {
            magic,
            version_major,
            version_minor,
            dir_offset,
            dir_len,
        };

        header.validate()?;
        Ok(header)
    }
}