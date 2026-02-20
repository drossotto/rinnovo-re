use std::io::{Read, Write};

pub const ATTR_TABLE_MAGIC: [u8; 4] = *b"ATB\0";
pub const ATTR_TABLE_VERSION_MAJOR: u16 = 0;
pub const ATTR_TABLE_VERSION_MINOR: u16 = 1;

/// Sparse attribute records attached to objects.
///
/// Each record is:
/// - object_id: u32  (row index into ObjectTable)
/// - key_sid:  u32  (StringDict ID for attribute key)
/// - value_sid: u32 (StringDict ID for attribute value)
/// - flags:    u32  (reserved / value type, currently 0)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeRecord {
    pub object_id: u32,
    pub key_sid: u32,
    pub value_sid: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeTable {
    pub attributes: Vec<AttributeRecord>,
}

impl AttributeTable {
    pub fn empty() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    pub fn push(&mut self, rec: AttributeRecord) {
        self.attributes.push(rec);
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&ATTR_TABLE_MAGIC)?;
        w.write_all(&ATTR_TABLE_VERSION_MAJOR.to_le_bytes())?;
        w.write_all(&ATTR_TABLE_VERSION_MINOR.to_le_bytes())?;

        let count: u32 = self.attributes.len().try_into().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "too many attributes for AttributeTable",
            )
        })?;
        w.write_all(&count.to_le_bytes())?;

        // reserved
        w.write_all(&0u32.to_le_bytes())?;

        for a in &self.attributes {
            w.write_all(&a.object_id.to_le_bytes())?;
            w.write_all(&a.key_sid.to_le_bytes())?;
            w.write_all(&a.value_sid.to_le_bytes())?;
            w.write_all(&a.flags.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != ATTR_TABLE_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid attribute table magic",
            ));
        }

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let vmaj = u16::from_le_bytes(buf2);
        r.read_exact(&mut buf2)?;
        let vmin = u16::from_le_bytes(buf2);

        if vmaj != ATTR_TABLE_VERSION_MAJOR || vmin != ATTR_TABLE_VERSION_MINOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported attribute table version",
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
                "attribute table reserved field must be 0",
            ));
        }

        let mut attributes = Vec::with_capacity(count);
        for _ in 0..count {
            r.read_exact(&mut buf4)?;
            let object_id = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let key_sid = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let value_sid = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let flags = u32::from_le_bytes(buf4);

            attributes.push(AttributeRecord {
                object_id,
                key_sid,
                value_sid,
                flags,
            });
        }

        Ok(Self { attributes })
    }
}

