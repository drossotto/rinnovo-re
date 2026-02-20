// crates/kernel/rnb_format/src/segment.rs

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Defines the type of structures that can be included in the RNB format. 
pub enum SegmentType {
    Manifest = 1,
    /// String dictionary segment holding deduplicated UTF‑8 strings.
    StringDict = 2,
}

impl SegmentType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(SegmentType::Manifest),
            2 => Some(SegmentType::StringDict),
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
}

impl QueryKernel {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(QueryKernel::GetObjectById),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}
