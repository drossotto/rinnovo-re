// crates/kernel/rnb_format/src/manifest.rs

use std::io::{Read, Write};

use crate::segment::{QueryKernel, SegmentType};

pub const MANIFEST_MAGIC: [u8; 4] = *b"MNF\0";
pub const MANIFEST_VERSION_MAJOR: u16 = 0;
pub const MANIFEST_VERSION_MINOR: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub flags: u32,
    pub required_segments: Vec<SegmentType>,
    pub supported_kernels: Vec<QueryKernel>,
    pub max_chunk_bytes: u32,
}

impl Manifest {
    pub fn minimal() -> Self {
        Self {
            flags: 0,
            required_segments: vec![SegmentType::Manifest],
            supported_kernels: Vec::new(),
            // A sane embedded-friendly default; you can tune later.
            max_chunk_bytes: 256 * 1024,
        }
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&MANIFEST_MAGIC)?;
        w.write_all(&MANIFEST_VERSION_MAJOR.to_le_bytes())?;
        w.write_all(&MANIFEST_VERSION_MINOR.to_le_bytes())?;

        w.write_all(&self.flags.to_le_bytes())?;

        let req_count: u32 = self.required_segments.len().try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "required_segments too large")
        })?;
        let ker_count: u32 = self.supported_kernels.len().try_into().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "supported_kernels too large")
        })?;

        w.write_all(&req_count.to_le_bytes())?;
        w.write_all(&ker_count.to_le_bytes())?;
        w.write_all(&self.max_chunk_bytes.to_le_bytes())?;
        w.write_all(&0u32.to_le_bytes())?; // reserved

        for s in &self.required_segments {
            w.write_all(&s.as_u32().to_le_bytes())?;
        }
        for k in &self.supported_kernels {
            w.write_all(&k.as_u32().to_le_bytes())?;
        }

        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != MANIFEST_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid manifest magic",
            ));
        }

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let vmaj = u16::from_le_bytes(buf2);
        r.read_exact(&mut buf2)?;
        let vmin = u16::from_le_bytes(buf2);

        if vmaj != MANIFEST_VERSION_MAJOR || vmin != MANIFEST_VERSION_MINOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported manifest version",
            ));
        }

        let mut buf4 = [0u8; 4];
        r.read_exact(&mut buf4)?;
        let flags = u32::from_le_bytes(buf4);

        r.read_exact(&mut buf4)?;
        let req_count = u32::from_le_bytes(buf4) as usize;

        r.read_exact(&mut buf4)?;
        let ker_count = u32::from_le_bytes(buf4) as usize;

        r.read_exact(&mut buf4)?;
        let max_chunk_bytes = u32::from_le_bytes(buf4);

        r.read_exact(&mut buf4)?;
        let reserved = u32::from_le_bytes(buf4);
        if reserved != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "manifest reserved field must be 0",
            ));
        }

        let mut required_segments = Vec::with_capacity(req_count);
        for _ in 0..req_count {
            r.read_exact(&mut buf4)?;
            let raw = u32::from_le_bytes(buf4);
            let st = SegmentType::from_u32(raw).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "unknown required segment")
            })?;
            required_segments.push(st);
        }

        let mut supported_kernels = Vec::with_capacity(ker_count);
        for _ in 0..ker_count {
            r.read_exact(&mut buf4)?;
            let raw = u32::from_le_bytes(buf4);
            let k = QueryKernel::from_u32(raw).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "unknown query kernel")
            })?;
            supported_kernels.push(k);
        }

        Ok(Self {
            flags,
            required_segments,
            supported_kernels,
            max_chunk_bytes,
        })
    }
}

/// Simple, dependency-free 64-bit FNV-1a checksum.
/// Not cryptographic — just a fast integrity check.
pub fn checksum64_fnv1a(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}