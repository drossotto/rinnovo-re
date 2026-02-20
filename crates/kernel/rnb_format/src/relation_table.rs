use std::io::{Read, Write};

pub const REL_TABLE_MAGIC: [u8; 4] = *b"REL\0";
pub const REL_TABLE_VERSION_MAJOR: u16 = 0;
pub const REL_TABLE_VERSION_MINOR: u16 = 1;

/// Relation records between objects.
///
/// Each record is:
/// - src_id: u32        (source object_id)
/// - dst_id: u32        (destination object_id)
/// - rel_type_sid: u32  (StringDict ID for relation type)
/// - flags: u32         (reserved)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationRecord {
    pub src_id: u32,
    pub dst_id: u32,
    pub rel_type_sid: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationTable {
    pub relations: Vec<RelationRecord>,
}

impl RelationTable {
    pub fn empty() -> Self {
        Self {
            relations: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.relations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.relations.is_empty()
    }

    pub fn push(&mut self, rec: RelationRecord) {
        self.relations.push(rec);
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&REL_TABLE_MAGIC)?;
        w.write_all(&REL_TABLE_VERSION_MAJOR.to_le_bytes())?;
        w.write_all(&REL_TABLE_VERSION_MINOR.to_le_bytes())?;

        let count: u32 = self.relations.len().try_into().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "too many relations for RelationTable",
            )
        })?;
        w.write_all(&count.to_le_bytes())?;

        // reserved
        w.write_all(&0u32.to_le_bytes())?;

        for rrec in &self.relations {
            w.write_all(&rrec.src_id.to_le_bytes())?;
            w.write_all(&rrec.dst_id.to_le_bytes())?;
            w.write_all(&rrec.rel_type_sid.to_le_bytes())?;
            w.write_all(&rrec.flags.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != REL_TABLE_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid relation table magic",
            ));
        }

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let vmaj = u16::from_le_bytes(buf2);
        r.read_exact(&mut buf2)?;
        let vmin = u16::from_le_bytes(buf2);

        if vmaj != REL_TABLE_VERSION_MAJOR || vmin != REL_TABLE_VERSION_MINOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported relation table version",
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
                "relation table reserved field must be 0",
            ));
        }

        let mut relations = Vec::with_capacity(count);
        for _ in 0..count {
            r.read_exact(&mut buf4)?;
            let src_id = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let dst_id = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let rel_type_sid = u32::from_le_bytes(buf4);

            r.read_exact(&mut buf4)?;
            let flags = u32::from_le_bytes(buf4);

            relations.push(RelationRecord {
                src_id,
                dst_id,
                rel_type_sid,
                flags,
            });
        }

        Ok(Self { relations })
    }
}

