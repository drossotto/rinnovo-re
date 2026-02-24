use std::collections::HashMap;
use std::io;

use crate::{artifact::RelationKernelArg, Artifact, QueryKernel, RelationRecord};

/// Semiring used to combine edge weights along a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemiringKind {
    /// Boolean reachability: any path contributes a 1.0 weight.
    Boolean,
    /// Sum-product: edge weights are multiplied along a path and summed
    /// over alternative paths. For v0 all edges are implicitly weight 1.0.
    SumProduct,
}

/// Specification of a typed relational path.
///
/// Each `rel_type_sids[i]` is a StringDict SID for a relation type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSpec {
    pub rel_type_sids: Vec<u32>,
    pub semiring: SemiringKind,
    pub state_id: Option<u32>,
}

/// Sparse CSR block over a subset of source and destination object IDs.
#[derive(Debug, Clone, PartialEq)]
pub struct SparseBlock {
    pub rows: u32,
    pub cols: u32,
    pub row_ids: Vec<u32>,
    pub col_ids: Vec<u32>,
    pub csr_indptr: Vec<u32>,
    pub csr_indices: Vec<u32>,
    pub data: Vec<f32>,
}

impl SparseBlock {
    pub fn empty() -> Self {
        Self {
            rows: 0,
            cols: 0,
            row_ids: Vec::new(),
            col_ids: Vec::new(),
            csr_indptr: vec![0],
            csr_indices: Vec::new(),
            data: Vec::new(),
        }
    }
}

impl Artifact {
    /// Project a path-constrained sparse block between `src_ids` and `dst_ids`.
    ///
    /// For each `src_id`, a frontier expansion is performed along the
    /// sequence of relation type SIDs in `spec.rel_type_sids`. The final
    /// frontier is intersected with `dst_ids` and emitted as a CSR row.
    pub fn project_path_block(
        &self,
        spec: &PathSpec,
        src_ids: &[u32],
        dst_ids: &[u32],
    ) -> io::Result<SparseBlock> {
        if spec.rel_type_sids.is_empty() || src_ids.is_empty() || dst_ids.is_empty() {
            return Ok(SparseBlock::empty());
        }

        // Map destination object IDs to column indices for quick lookup.
        let mut dst_index: HashMap<u32, u32> = HashMap::with_capacity(dst_ids.len());
        for (i, id) in dst_ids.iter().enumerate() {
            dst_index.insert(*id, i as u32);
        }

        let rows = src_ids.len() as u32;
        let cols = dst_ids.len() as u32;

        let mut csr_indptr = Vec::with_capacity(src_ids.len() + 1);
        let mut csr_indices: Vec<u32> = Vec::new();
        let mut data: Vec<f32> = Vec::new();

        csr_indptr.push(0);

        for &src_id in src_ids {
            // Frontier: node_id -> weight.
            let mut frontier: HashMap<u32, f32> = HashMap::new();
            frontier.insert(src_id, 1.0);

            for &rel_sid in &spec.rel_type_sids {
                let mut next: HashMap<u32, f32> = HashMap::new();

                for (&node, &weight) in frontier.iter() {
                    let records: Vec<RelationRecord> = self.execute_relations(
                        QueryKernel::GetRelationsFrom,
                        RelationKernelArg {
                            id: node,
                            rel_type_sid: Some(rel_sid),
                        },
                    )?;

                    if records.is_empty() {
                        continue;
                    }

                    for r in records {
                        let neighbor = r.dst_id;
                        let edge_weight = 1.0f32;
                        let contrib = match spec.semiring {
                            SemiringKind::Boolean => 1.0,
                            SemiringKind::SumProduct => weight * edge_weight,
                        };

                        next.entry(neighbor).and_modify(|w| {
                            match spec.semiring {
                                SemiringKind::Boolean => {
                                    if *w < 1.0 {
                                        *w = 1.0;
                                    }
                                }
                                SemiringKind::SumProduct => {
                                    *w += contrib;
                                }
                            }
                        }).or_insert(contrib);
                    }
                }

                frontier = next;
                if frontier.is_empty() {
                    break;
                }
            }

            // Emit CSR entries for this row.
            let mut entries: Vec<(u32, f32)> = Vec::new();
            for (node, &weight) in frontier.iter() {
                if let Some(&col_idx) = dst_index.get(node) {
                    let v = match spec.semiring {
                        SemiringKind::Boolean => 1.0,
                        SemiringKind::SumProduct => weight,
                    };
                    entries.push((col_idx, v));
                }
            }

            // Keep column indices sorted for deterministic representations.
            entries.sort_by_key(|(c, _)| *c);

            for (col_idx, v) in entries {
                csr_indices.push(col_idx);
                data.push(v);
            }

            csr_indptr.push(csr_indices.len() as u32);
        }

        Ok(SparseBlock {
            rows,
            cols,
            row_ids: src_ids.to_vec(),
            col_ids: dst_ids.to_vec(),
            csr_indptr,
            csr_indices,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AttributeTable, ConstraintDef, EdgeTypeDef, Manifest, NodeTypeDef, ObjectRecord, ObjectTable,
        RelationRecord, RelationTable, RnbDirectory, RnbFile, RnbHeader, SegmentType, StringDict,
        TypeRegistry,
    };

    /// Build a tiny artifact with:
    /// cell(0) --relA--> gene(1) --relA--> protein(2)
    fn make_chain_artifact() -> Artifact {
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

        // StringDict: node type labels + relation type label.
        // 0 -> "cell", 1 -> "gene", 2 -> "protein", 3 -> "relA"
        let string_dict = Some(StringDict::new(vec![
            "cell".to_string(),
            "gene".to_string(),
            "protein".to_string(),
            "relA".to_string(),
        ]));

        // Objects: three nodes with StringDict-based type_sids.
        let mut ot = ObjectTable::empty();
        ot.push(ObjectRecord {
            type_sid: 0,
            name_sid: 0,
            flags: 0,
        }); // id 0: cell
        ot.push(ObjectRecord {
            type_sid: 1,
            name_sid: 1,
            flags: 0,
        }); // id 1: gene
        ot.push(ObjectRecord {
            type_sid: 2,
            name_sid: 2,
            flags: 0,
        }); // id 2: protein
        let object_table = Some(ot);

        // Relations: 0->1 and 1->2, both rel_type_sid = 3 ("relA").
        let mut rt = RelationTable::empty();
        rt.push(RelationRecord {
            src_id: 0,
            dst_id: 1,
            rel_type_sid: 3,
            flags: 0,
        });
        rt.push(RelationRecord {
            src_id: 1,
            dst_id: 2,
            rel_type_sid: 3,
            flags: 0,
        });
        let relation_table = Some(rt);

        // Type registry: node types and an edge type for relA: cell -> gene and gene -> protein
        // For this test we only need adjacency validation, which is already covered
        // elsewhere; here we just ensure the registry exists.
        let registry = TypeRegistry {
            schema_version: 1,
            node_types: vec![
                NodeTypeDef {
                    type_id: 1,
                    name_sid: 0,
                }, // cell
                NodeTypeDef {
                    type_id: 2,
                    name_sid: 1,
                }, // gene
                NodeTypeDef {
                    type_id: 3,
                    name_sid: 2,
                }, // protein
            ],
            edge_types: vec![EdgeTypeDef {
                type_id: 10,
                name_sid: 3, // relA
                src_node_type_id: 1,
                dst_node_type_id: 2,
            }],
            constraints: vec![ConstraintDef::TypeAdjacency {
                edge_type_id: 10,
                src_node_type_id: 1,
                dst_node_type_id: 2,
            }],
        };

        let inner = RnbFile {
            header,
            directory,
            manifest,
            string_dict,
            object_table,
            attribute_table: None as Option<AttributeTable>,
            relation_table,
            type_registry: Some(registry),
            numeric_matrix: None,
            sparse_matrix: None,
        };

        Artifact::from_rnb_file(inner)
    }

    #[test]
    fn project_path_block_two_step_chain_boolean() {
        let art = make_chain_artifact();

        // Path: relA, relA (two hops)
        let spec = PathSpec {
            rel_type_sids: vec![3, 3],
            semiring: SemiringKind::Boolean,
            state_id: None,
        };

        let src_ids = [0u32];
        let dst_ids = [2u32];

        let block = art.project_path_block(&spec, &src_ids, &dst_ids).unwrap();
        assert_eq!(block.rows, 1);
        assert_eq!(block.cols, 1);
        assert_eq!(block.row_ids, vec![0]);
        assert_eq!(block.col_ids, vec![2]);
        assert_eq!(block.csr_indptr, vec![0, 1]);
        assert_eq!(block.csr_indices, vec![0]); // col index 0 corresponds to dst_id 2
        assert_eq!(block.data, vec![1.0]);
    }
}
