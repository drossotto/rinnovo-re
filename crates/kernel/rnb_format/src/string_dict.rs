// crates/kernel/rnb_format/src/string_dict.rs

use std::io::{Read, Write};

pub const STRDICT_MAGIC: [u8; 4] = *b"SDCT";
pub const STRDICT_VERSION_MAJOR: u16 = 0;
pub const STRDICT_VERSION_MINOR: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringDict {
    pub strings: Vec<String>,
}

impl StringDict {
    pub fn new(strings: Vec<String>) -> Self {
        Self { strings }
    }

    pub fn empty() -> Self {
        Self { strings: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn get(&self, id: u32) -> Option<&str> {
        self.strings.get(id as usize).map(|s| s.as_str())
    }

    pub fn push(&mut self, s: String) -> u32 {
        let id = self.strings.len() as u32;
        self.strings.push(s);
        id
    }

    /// Serialize into bytes (so caller can checksum and write once).
    pub fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        // Precompute utf8 bytes and offsets
        let mut blob: Vec<u8> = Vec::new();
        let mut offsets: Vec<u32> = Vec::with_capacity(self.strings.len() + 1);

        offsets.push(0);
        for s in &self.strings {
            let b = s.as_bytes();
            // Bound: offsets are u32 for compactness; enforce.
            if blob.len() + b.len() > (u32::MAX as usize) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "string dict blob too large for u32 offsets",
                ));
            }
            blob.extend_from_slice(b);
            offsets.push(blob.len() as u32);
        }

        let count: u32 = self.strings.len().try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "too many strings")
        })?;

        let blob_len: u32 = blob.len() as u32;

        let mut out: Vec<u8> = Vec::new();
        out.write_all(&STRDICT_MAGIC)?;
        out.write_all(&STRDICT_VERSION_MAJOR.to_le_bytes())?;
        out.write_all(&STRDICT_VERSION_MINOR.to_le_bytes())?;
        out.write_all(&count.to_le_bytes())?;
        out.write_all(&blob_len.to_le_bytes())?;

        // offsets table (count+1) u32s
        for off in offsets {
            out.write_all(&off.to_le_bytes())?;
        }

        // blob
        out.write_all(&blob)?;

        Ok(out)
    }

    pub fn from_bytes(bytes: &[u8]) -> std::io::Result<Self> {
        let mut r = std::io::Cursor::new(bytes);

        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != STRDICT_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid string dict magic",
            ));
        }

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let vmaj = u16::from_le_bytes(buf2);
        r.read_exact(&mut buf2)?;
        let vmin = u16::from_le_bytes(buf2);
        if vmaj != STRDICT_VERSION_MAJOR || vmin != STRDICT_VERSION_MINOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported string dict version",
            ));
        }

        let mut buf4 = [0u8; 4];
        r.read_exact(&mut buf4)?;
        let count = u32::from_le_bytes(buf4) as usize;

        r.read_exact(&mut buf4)?;
        let blob_len = u32::from_le_bytes(buf4) as usize;

        // offsets
        let mut offsets: Vec<u32> = Vec::with_capacity(count + 1);
        for _ in 0..(count + 1) {
            r.read_exact(&mut buf4)?;
            offsets.push(u32::from_le_bytes(buf4));
        }

        // Remaining bytes must contain blob_len bytes
        let pos = r.position() as usize;
        if bytes.len() < pos + blob_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "string dict truncated blob",
            ));
        }
        let blob = &bytes[pos..pos + blob_len];

        // Validate offsets monotonic and final offset == blob_len
        if offsets.first().copied().unwrap_or(1) != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "string dict offsets must start at 0",
            ));
        }
        if offsets.last().copied().unwrap_or(0) as usize != blob_len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "string dict last offset must equal blob_len",
            ));
        }
        for w in offsets.windows(2) {
            if w[0] > w[1] {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "string dict offsets must be non-decreasing",
                ));
            }
        }

        let mut strings: Vec<String> = Vec::with_capacity(count);
        for i in 0..count {
            let a = offsets[i] as usize;
            let b = offsets[i + 1] as usize;
            let slice = &blob[a..b];
            let s = std::str::from_utf8(slice).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid utf8 in dict")
            })?;
            strings.push(s.to_string());
        }

        Ok(Self { strings })
    }
}
