// crates/kernel/rnb_format/src/segment.rs

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Defines the type of structures that can be included in the RNB format.
pub enum SegmentType {
    Manifest = 1,
    /// String dictionary segment holding deduplicated UTF-8 strings.
    StringDict = 2,
    /// Object table mapping object IDs to basic metadata.
    ObjectTable = 3,
    /// Attribute table storing sparse key/value metadata.
    AttributeTable = 4,
    /// Relation table storing edges between objects.
    RelationTable = 5,
    /// Dense numeric matrix payload.
    NumericMatrix = 6,
    /// Type registry describing canonical node/edge types and constraints.
    TypeRegistry = 7,
    /// Sparse numeric matrix payload (CSR/CSC).
    SparseMatrix = 8,
}

impl SegmentType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(SegmentType::Manifest),
            2 => Some(SegmentType::StringDict),
            3 => Some(SegmentType::ObjectTable),
            4 => Some(SegmentType::AttributeTable),
            5 => Some(SegmentType::RelationTable),
            6 => Some(SegmentType::NumericMatrix),
            7 => Some(SegmentType::TypeRegistry),
            8 => Some(SegmentType::SparseMatrix),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryKernel {
    GetObjectById = 1,
    ObjectsByType = 2,
    GetRelationsFrom = 3,
    GetRelationsTo = 4,
}

impl QueryKernel {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(QueryKernel::GetObjectById),
            2 => Some(QueryKernel::ObjectsByType),
            3 => Some(QueryKernel::GetRelationsFrom),
            4 => Some(QueryKernel::GetRelationsTo),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}
