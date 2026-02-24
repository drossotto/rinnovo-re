use std::io::{Read, Write};

pub const TYPE_REGISTRY_MAGIC: [u8; 4] = *b"TRG\0";
pub const TYPE_REGISTRY_VERSION_MAJOR: u16 = 0;
pub const TYPE_REGISTRY_VERSION_MINOR: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeTypeDef {
    pub type_id: u32,
    pub name_sid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeTypeDef {
    pub type_id: u32,
    pub name_sid: u32,
    pub src_node_type_id: u32,
    pub dst_node_type_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintDef {
    /// Enforce that edges of `edge_type_id` always connect the given
    /// source and destination node types.
    TypeAdjacency {
        edge_type_id: u32,
        src_node_type_id: u32,
        dst_node_type_id: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRegistry {
    pub schema_version: u32,
    pub node_types: Vec<NodeTypeDef>,
    pub edge_types: Vec<EdgeTypeDef>,
    pub constraints: Vec<ConstraintDef>,
}

impl TypeRegistry {
    pub fn new(schema_version: u32) -> Self {
        Self {
            schema_version,
            node_types: Vec::new(),
            edge_types: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&TYPE_REGISTRY_MAGIC)?;
        w.write_all(&TYPE_REGISTRY_VERSION_MAJOR.to_le_bytes())?;
        w.write_all(&TYPE_REGISTRY_VERSION_MINOR.to_le_bytes())?;

        w.write_all(&self.schema_version.to_le_bytes())?;

        let node_count: u32 = self
            .node_types
            .len()
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "too many node types"))?;
        let edge_count: u32 = self
            .edge_types
            .len()
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "too many edge types"))?;
        let constraint_count: u32 = self
            .constraints
            .len()
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "too many constraints"))?;

        w.write_all(&node_count.to_le_bytes())?;
        w.write_all(&edge_count.to_le_bytes())?;
        w.write_all(&constraint_count.to_le_bytes())?;
        w.write_all(&0u32.to_le_bytes())?; // reserved

        for n in &self.node_types {
            w.write_all(&n.type_id.to_le_bytes())?;
            w.write_all(&n.name_sid.to_le_bytes())?;
        }

        for e in &self.edge_types {
            w.write_all(&e.type_id.to_le_bytes())?;
            w.write_all(&e.name_sid.to_le_bytes())?;
            w.write_all(&e.src_node_type_id.to_le_bytes())?;
            w.write_all(&e.dst_node_type_id.to_le_bytes())?;
        }

        for c in &self.constraints {
            match *c {
                ConstraintDef::TypeAdjacency {
                    edge_type_id,
                    src_node_type_id,
                    dst_node_type_id,
                } => {
                    // Variant tag for TypeAdjacency.
                    w.write_all(&1u32.to_le_bytes())?;
                    w.write_all(&edge_type_id.to_le_bytes())?;
                    w.write_all(&src_node_type_id.to_le_bytes())?;
                    w.write_all(&dst_node_type_id.to_le_bytes())?;
                }
            }
        }

        Ok(())
    }

    pub fn read_from<R: Read>(mut r: R) -> std::io::Result<Self> {
        let mut magic = [0u8; 4];
        r.read_exact(&mut magic)?;
        if magic != TYPE_REGISTRY_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid type registry magic",
            ));
        }

        let mut buf2 = [0u8; 2];
        r.read_exact(&mut buf2)?;
        let vmaj = u16::from_le_bytes(buf2);
        r.read_exact(&mut buf2)?;
        let vmin = u16::from_le_bytes(buf2);
        if vmaj != TYPE_REGISTRY_VERSION_MAJOR || vmin != TYPE_REGISTRY_VERSION_MINOR {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unsupported type registry version",
            ));
        }

        let mut buf4 = [0u8; 4];
        r.read_exact(&mut buf4)?;
        let schema_version = u32::from_le_bytes(buf4);

        r.read_exact(&mut buf4)?;
        let node_count = u32::from_le_bytes(buf4) as usize;
        r.read_exact(&mut buf4)?;
        let edge_count = u32::from_le_bytes(buf4) as usize;
        r.read_exact(&mut buf4)?;
        let constraint_count = u32::from_le_bytes(buf4) as usize;

        r.read_exact(&mut buf4)?;
        let reserved = u32::from_le_bytes(buf4);
        if reserved != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "type registry reserved field must be 0",
            ));
        }

        let mut node_types = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            r.read_exact(&mut buf4)?;
            let type_id = u32::from_le_bytes(buf4);
            r.read_exact(&mut buf4)?;
            let name_sid = u32::from_le_bytes(buf4);
            node_types.push(NodeTypeDef { type_id, name_sid });
        }

        let mut edge_types = Vec::with_capacity(edge_count);
        for _ in 0..edge_count {
            r.read_exact(&mut buf4)?;
            let type_id = u32::from_le_bytes(buf4);
            r.read_exact(&mut buf4)?;
            let name_sid = u32::from_le_bytes(buf4);
            r.read_exact(&mut buf4)?;
            let src_node_type_id = u32::from_le_bytes(buf4);
            r.read_exact(&mut buf4)?;
            let dst_node_type_id = u32::from_le_bytes(buf4);
            edge_types.push(EdgeTypeDef {
                type_id,
                name_sid,
                src_node_type_id,
                dst_node_type_id,
            });
        }

        let mut constraints = Vec::with_capacity(constraint_count);
        for _ in 0..constraint_count {
            r.read_exact(&mut buf4)?;
            let tag = u32::from_le_bytes(buf4);
            match tag {
                1 => {
                    r.read_exact(&mut buf4)?;
                    let edge_type_id = u32::from_le_bytes(buf4);
                    r.read_exact(&mut buf4)?;
                    let src_node_type_id = u32::from_le_bytes(buf4);
                    r.read_exact(&mut buf4)?;
                    let dst_node_type_id = u32::from_le_bytes(buf4);
                    constraints.push(ConstraintDef::TypeAdjacency {
                        edge_type_id,
                        src_node_type_id,
                        dst_node_type_id,
                    });
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "unknown type registry constraint tag",
                    ));
                }
            }
        }

        Ok(Self {
            schema_version,
            node_types,
            edge_types,
            constraints,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_type_registry() {
        let mut reg = TypeRegistry::new(1);
        reg.node_types.push(NodeTypeDef {
            type_id: 1,
            name_sid: 10,
        });
        reg.edge_types.push(EdgeTypeDef {
            type_id: 2,
            name_sid: 20,
            src_node_type_id: 1,
            dst_node_type_id: 1,
        });
        reg.constraints.push(ConstraintDef::TypeAdjacency {
            edge_type_id: 2,
            src_node_type_id: 1,
            dst_node_type_id: 1,
        });

        let mut buf = Vec::new();
        reg.write_to(&mut buf).unwrap();

        let decoded = TypeRegistry::read_from(&buf[..]).unwrap();
        assert_eq!(reg, decoded);
    }
}

