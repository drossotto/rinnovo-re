use std::io::{Read, Write};

pub const OBJECT_TABLE_MAGIC: [u8; 4] = *b"OBT\0";
pub const OBJECT_TABLE_VERSION_MAJOR: u16 = 0;
pub const OBJECT_TABLE_VERSION_MINOR: u16 = 1;

/// A minimal, fixed-width object table segment.
///
/// This is intentionally small and generic: each row corresponds to a
/// logical object_id (its index in `objects`) and stores:
///
/// - `type_sid`: u32  (StringDict ID for the object's type/kind)
/// - `name_sid`: u32  (StringDict ID for a primary label/name)
/// - `flags`:   u32  (reserved for future use)
///
/// All richer semantics (attributes, relationships, payloads) will be
/// layered on top in later commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectRecord {
    pub type_sid: u32,
    pub name_sid: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectTable {
    pub objects: Vec<ObjectRecord>,
}

impl ObjectTable {
    pub fn empty() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn push(&mut self, rec: ObjectRecord) {
        self.objects.push(rec);
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&OBJECT_TABLE_MAGIC)?;
        w.write_all(&OBJECT_TABLE_VERSION_MAJOR.to_le_bytes())?;
        w.write_all(&OBJECT_TABLE_VERSION_MINOR.to_le_bytes())?;

        let count: u32 = self.objects.len().try_into().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "too many objects for ObjectTable",
            )
        })?;
        w.write_all(&count.to_le_bytes())?;

        // reserved
        w.write_all(&0u32.to_le_bytes())?;

        for o in &self.objects {
            w.write_all(&o.type_sid.to_le_bytes())?;
            w.write_all(&o.name_sid.to_le_bytes())?;
            w.write_all(&o.flags.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != OBJECT_TABLE_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid object table magic",
            ));
        }

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let vmaj = u16::from_le_bytes(buf2);
        r.read_exact(&mut buf2)?;
        let vmin = u16::from_le_bytes(buf2);

        if vmaj != OBJECT_TABLE_VERSION_MAJOR || vmin != OBJECT_TABLE_VERSION_MINOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported object table version",
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
                "object table reserved field must be 0",
            ));
        }

        let mut objects = Vec::with_capacity(count);
        for _ in 0..count {
            r.read_exact(&mut buf4)?;
            let type_sid = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let name_sid = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let flags = u32::from_le_bytes(buf4);

            objects.push(ObjectRecord {
                type_sid,
                name_sid,
                flags,
            });
        }

        Ok(Self { objects })
    }
}

