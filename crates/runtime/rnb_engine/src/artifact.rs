use std::path::Path;
use std::collections::HashMap;

use crate::{
    AttributeTable,
    Manifest,
    ObjectTable,
    RelationTable,
    RnbDirectory,
    RnbFile,
    RnbHeader,
    NumericMatrix,
    StringDict,
    TypeRegistry,
};

/// High-level wrapper around a parsed RNB artifact.
///
/// This type exposes a stable, semantic view over the underlying
/// `rnb_format::RnbFile` while hiding the low-level I/O details.
#[derive(Debug, Clone)]
pub struct Artifact {
    inner: RnbFile,
}

/// Arguments for relation kernels executed via `Artifact::execute_relations`.
#[derive(Debug, Clone, Copy)]
pub struct RelationKernelArg {
    pub id: u32,
    pub rel_type_sid: Option<u32>,
}

impl Artifact {
    /// Internal helper used by tests to construct an `Artifact` from an
    /// already-populated `RnbFile` without going through the on-disk
    /// parser. This is only compiled in test builds.
    #[cfg(test)]
    pub(crate) fn from_rnb_file(inner: RnbFile) -> Self {
        Self { inner }
    }
    /// Open an RNB artifact from the given path.
    ///
    /// This method validates the header, directory, manifest, and any
    /// invariants enforced by the container (e.g. required segments).
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = rnb_format::open_rnb(path)?;
        let art = Self { inner: file };
        art.validate_schema_invariants()?;
        Ok(art)
    }

    /// Borrow the underlying raw representation.
    pub fn as_rnb_file(&self) -> &RnbFile {
        &self.inner
    }

    pub fn header(&self) -> &RnbHeader {
        &self.inner.header
    }

    pub fn directory(&self) -> &RnbDirectory {
        &self.inner.directory
    }

    pub fn manifest(&self) -> &Manifest {
        &self.inner.manifest
    }

    pub fn string_dict(&self) -> Option<&StringDict> {
        self.inner.string_dict.as_ref()
    }

    pub fn object_table(&self) -> Option<&ObjectTable> {
        self.inner.object_table.as_ref()
    }

    pub fn attribute_table(&self) -> Option<&AttributeTable> {
        self.inner.attribute_table.as_ref()
    }

    pub fn relation_table(&self) -> Option<&RelationTable> {
        self.inner.relation_table.as_ref()
    }

    /// Return the optional type registry segment, if present.
    pub fn type_registry(&self) -> Option<&TypeRegistry> {
        self.inner.type_registry.as_ref()
    }

    /// Return the first numeric matrix segment, if present.
    ///
    /// The container format allows multiple NumericMatrix segments,
    /// but the engine currently exposes a single primary matrix view.
    pub fn numeric_matrix(&self) -> Option<&NumericMatrix> {
        self.inner.numeric_matrix.as_ref()
    }

    /// Validate any schema invariants declared by the type registry.
    ///
    /// For v0 this is a no-op unless a `TypeRegistry` is present;
    /// even then, the implementation is intentionally conservative
    /// and may evolve over time. Callers should treat any error as
    /// a hard validation failure, but must not rely on all possible
    /// issues being detected yet.
    pub fn validate_schema_invariants(&self) -> std::io::Result<()> {
        use std::io::{Error, ErrorKind};

        let registry = match self.type_registry() {
            Some(tr) => tr,
            None => return Ok(()),
        };

        let relations = match self.relation_table() {
            Some(rt) => rt,
            None => return Ok(()),
        };

        let objects = match self.object_table() {
            Some(ot) => ot,
            None => return Ok(()),
        };

        // Map relation type SID -> edge_type_id.
        let mut rel_sid_to_edge_type: HashMap<u32, u32> = HashMap::new();
        for e in &registry.edge_types {
            rel_sid_to_edge_type.insert(e.name_sid, e.type_id);
        }

        // Map object type SID -> node_type_id.
        let mut obj_type_sid_to_node_type: HashMap<u32, u32> = HashMap::new();
        for n in &registry.node_types {
            obj_type_sid_to_node_type.insert(n.name_sid, n.type_id);
        }

        // Map edge_type_id -> (src_node_type_id, dst_node_type_id) for
        // TypeAdjacency constraints. At the moment this is the only
        // constraint variant, so we destructure directly.
        let mut edge_constraints: HashMap<u32, (u32, u32)> = HashMap::new();
        for c in &registry.constraints {
            let crate::ConstraintDef::TypeAdjacency {
                edge_type_id,
                src_node_type_id,
                dst_node_type_id,
            } = *c;
            edge_constraints.insert(edge_type_id, (src_node_type_id, dst_node_type_id));
        }

        if edge_constraints.is_empty() {
            // Nothing to enforce yet.
            return Ok(());
        }

        for rel in &relations.relations {
            let edge_type_id = match rel_sid_to_edge_type.get(&rel.rel_type_sid) {
                Some(id) => *id,
                None => continue, // relation type not covered by registry; skip
            };

            let (expected_src_type, expected_dst_type) = match edge_constraints.get(&edge_type_id)
            {
                Some(pair) => *pair,
                None => continue, // no adjacency constraint for this edge type
            };

            let src_row = objects.objects.get(rel.src_id as usize).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    "relation src_id out of bounds for ObjectTable",
                )
            })?;
            let dst_row = objects.objects.get(rel.dst_id as usize).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    "relation dst_id out of bounds for ObjectTable",
                )
            })?;

            let src_node_type = match obj_type_sid_to_node_type.get(&src_row.type_sid) {
                Some(id) => *id,
                None => continue, // object type not covered; skip
            };
            let dst_node_type = match obj_type_sid_to_node_type.get(&dst_row.type_sid) {
                Some(id) => *id,
                None => continue,
            };

            if src_node_type != expected_src_type || dst_node_type != expected_dst_type {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "type adjacency constraint violated",
                ));
            }
        }

        Ok(())
    }

    /// Return all attribute records attached to the given `object_id`.
    ///
    /// The result is empty if there is no AttributeTable segment or
    /// if the object has no attributes.
    pub fn attributes_for_object(
        &self,
        object_id: u32,
    ) -> Option<impl Iterator<Item = &crate::AttributeRecord>> {
        let table = self.attribute_table()?;
        Some(table.attributes.iter().filter(move |a| a.object_id == object_id))
    }

    /// Return all relations originating from the given `src_id`.
    ///
    /// If `rel_type_sid` is `Some`, the results are further filtered
    /// to only that relation type.
    pub fn relations_from(
        &self,
        src_id: u32,
        rel_type_sid: Option<u32>,
    ) -> Option<impl Iterator<Item = &crate::RelationRecord>> {
        let table = self.relation_table()?;
        Some(table.relations.iter().filter(move |r| {
            r.src_id == src_id
                && rel_type_sid.map(|sid| r.rel_type_sid == sid).unwrap_or(true)
        }))
    }

    /// Return all relations targeting the given `dst_id`.
    ///
    /// If `rel_type_sid` is `Some`, the results are further filtered
    /// to only that relation type.
    pub fn relations_to(
        &self,
        dst_id: u32,
        rel_type_sid: Option<u32>,
    ) -> Option<impl Iterator<Item = &crate::RelationRecord>> {
        let table = self.relation_table()?;
        Some(table.relations.iter().filter(move |r| {
            r.dst_id == dst_id
                && rel_type_sid.map(|sid| r.rel_type_sid == sid).unwrap_or(true)
        }))
    }

    /// Execute a relation-level query kernel against this artifact.
    ///
    /// This mirrors the manifest-based dispatch of object kernels but
    /// returns `RelationRecord` values instead of logical `Object`
    /// views. Only relation kernels are accepted.
    pub fn execute_relations(
        &self,
        kernel: crate::QueryKernel,
        arg: RelationKernelArg,
    ) -> std::io::Result<Vec<crate::RelationRecord>> {
        use std::io::{Error, ErrorKind};

        match kernel {
            crate::QueryKernel::GetRelationsFrom | crate::QueryKernel::GetRelationsTo => {}
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "kernel is not a relation kernel",
                ));
            }
        }

        let manifest = self.manifest();
        if !manifest.supported_kernels.is_empty()
            && !manifest.supported_kernels.contains(&kernel)
        {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "query kernel not supported by this artifact",
            ));
        }

        let table = match self.relation_table() {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        let mut out = Vec::new();
        match kernel {
            crate::QueryKernel::GetRelationsFrom => {
                for r in &table.relations {
                    if r.src_id == arg.id
                        && arg
                            .rel_type_sid
                            .map(|sid| r.rel_type_sid == sid)
                            .unwrap_or(true)
                    {
                        out.push(r.clone());
                    }
                }
            }
            crate::QueryKernel::GetRelationsTo => {
                for r in &table.relations {
                    if r.dst_id == arg.id
                        && arg
                            .rel_type_sid
                            .map(|sid| r.rel_type_sid == sid)
                            .unwrap_or(true)
                    {
                        out.push(r.clone());
                    }
                }
            }
            _ => unreachable!(),
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ConstraintDef, EdgeTypeDef, Manifest, NodeTypeDef, ObjectRecord, ObjectTable, QueryKernel,
        RelationRecord, RelationTable, RnbDirectory, RnbFile, RnbHeader, SegmentType, StringDict,
        TypeRegistry,
    };

    fn make_test_artifact_with_relations() -> Artifact {
        let header = RnbHeader::new();
        let directory = RnbDirectory::empty();

        let manifest = Manifest {
            flags: 0,
            required_segments: vec![SegmentType::Manifest],
            supported_kernels: vec![
                QueryKernel::GetRelationsFrom,
                QueryKernel::GetRelationsTo,
            ],
            max_chunk_bytes: 256 * 1024,
        };

        // Minimal string dict / object table just so the RnbFile looks
        // structurally reasonable. The relation kernels do not depend
        // on their contents yet.
        let string_dict = Some(StringDict::new(vec!["rel_type".to_string()]));

        let mut ot = ObjectTable::empty();
        ot.push(ObjectRecord {
            type_sid: 0,
            name_sid: 0,
            flags: 0,
        });
        ot.push(ObjectRecord {
            type_sid: 0,
            name_sid: 0,
            flags: 0,
        });
        let object_table = Some(ot);

        let mut rt = RelationTable::empty();
        rt.push(RelationRecord {
            src_id: 0,
            dst_id: 1,
            rel_type_sid: 0,
            flags: 0,
        });
        rt.push(RelationRecord {
            src_id: 1,
            dst_id: 0,
            rel_type_sid: 0,
            flags: 0,
        });
        let relation_table = Some(rt);

        let inner = RnbFile {
            header,
            directory,
            manifest,
            string_dict,
            object_table,
            attribute_table: None,
            relation_table,
            type_registry: None,
            numeric_matrix: None,
            sparse_matrix: None,
        };

        Artifact { inner }
    }

    #[test]
    fn execute_relations_from_and_to_kernels() {
        let art = make_test_artifact_with_relations();

        // From src_id = 0
        let from = art
            .execute_relations(
                QueryKernel::GetRelationsFrom,
                RelationKernelArg {
                    id: 0,
                    rel_type_sid: None,
                },
            )
            .unwrap();
        assert_eq!(from.len(), 1);
        assert_eq!(from[0].src_id, 0);
        assert_eq!(from[0].dst_id, 1);

        // To dst_id = 0
        let to = art
            .execute_relations(
                QueryKernel::GetRelationsTo,
                RelationKernelArg {
                    id: 0,
                    rel_type_sid: None,
                },
            )
            .unwrap();
        assert_eq!(to.len(), 1);
        assert_eq!(to[0].src_id, 1);
        assert_eq!(to[0].dst_id, 0);
    }

    #[test]
    fn validate_schema_invariants_type_adjacency() {
        // Node types: 1 = cell, 2 = gene
        let registry = TypeRegistry {
            schema_version: 1,
            node_types: vec![
                NodeTypeDef {
                    type_id: 1,
                    name_sid: 0, // "cell"
                },
                NodeTypeDef {
                    type_id: 2,
                    name_sid: 1, // "gene"
                },
            ],
            edge_types: vec![EdgeTypeDef {
                type_id: 10,
                name_sid: 0, // same SID as relation type below
                src_node_type_id: 1,
                dst_node_type_id: 2,
            }],
            constraints: vec![ConstraintDef::TypeAdjacency {
                edge_type_id: 10,
                src_node_type_id: 1,
                dst_node_type_id: 2,
            }],
        };

        let header = RnbHeader::new();
        let directory = RnbDirectory::empty();
        let manifest = Manifest {
            flags: 0,
            required_segments: vec![SegmentType::Manifest],
            supported_kernels: Vec::new(),
            max_chunk_bytes: 256 * 1024,
        };

        let string_dict = Some(StringDict::new(vec![
            "cell".to_string(), // sid 0
            "gene".to_string(), // sid 1
        ]));

        let mut ot = ObjectTable::empty();
        ot.push(ObjectRecord {
            type_sid: 0,
            name_sid: 0,
            flags: 0,
        }); // cell
        ot.push(ObjectRecord {
            type_sid: 1,
            name_sid: 1,
            flags: 0,
        }); // gene

        let object_table = Some(ot);

        let mut rt = RelationTable::empty();
        rt.push(RelationRecord {
            src_id: 0,
            dst_id: 1,
            rel_type_sid: 0, // matches edge_types[0].name_sid
            flags: 0,
        });

        let relation_table = Some(rt);

        let inner_ok = RnbFile {
            header,
            directory,
            manifest,
            string_dict,
            object_table,
            attribute_table: None,
            relation_table,
            type_registry: Some(registry.clone()),
            numeric_matrix: None,
            sparse_matrix: None,
        };

        let art_ok = Artifact { inner: inner_ok };
        art_ok.validate_schema_invariants().unwrap();

        // Now make a violating artifact: flip the dst object type to "cell".
        let header_bad = RnbHeader::new();
        let directory_bad = RnbDirectory::empty();
        let manifest_bad = Manifest {
            flags: 0,
            required_segments: vec![SegmentType::Manifest],
            supported_kernels: Vec::new(),
            max_chunk_bytes: 256 * 1024,
        };

        let string_dict_bad = Some(StringDict::new(vec![
            "cell".to_string(), // sid 0
            "gene".to_string(), // sid 1
        ]));

        let mut ot_bad = ObjectTable::empty();
        ot_bad.push(ObjectRecord {
            type_sid: 0,
            name_sid: 0,
            flags: 0,
        }); // cell
        ot_bad.push(ObjectRecord {
            type_sid: 0,
            name_sid: 0,
            flags: 0,
        }); // also cell (should be gene)

        let object_table_bad = Some(ot_bad);

        let mut rt_bad = RelationTable::empty();
        rt_bad.push(RelationRecord {
            src_id: 0,
            dst_id: 1,
            rel_type_sid: 0,
            flags: 0,
        });
        let relation_table_bad = Some(rt_bad);

        let inner_bad = RnbFile {
            header: header_bad,
            directory: directory_bad,
            manifest: manifest_bad,
            string_dict: string_dict_bad,
            object_table: object_table_bad,
            attribute_table: None,
            relation_table: relation_table_bad,
            type_registry: Some(registry),
            numeric_matrix: None,
            sparse_matrix: None,
        };

        let art_bad = Artifact { inner: inner_bad };
        let err = art_bad.validate_schema_invariants().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
