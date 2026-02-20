use crate::artifact::Artifact;

/// Logical object view derived from the ObjectTable segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    pub id: u32,
    pub type_sid: u32,
    pub name_sid: u32,
    pub flags: u32,
}

impl Object {
    pub fn from_row(id: u32, row: &crate::ObjectRecord) -> Self {
        Self {
            id,
            type_sid: row.type_sid,
            name_sid: row.name_sid,
            flags: row.flags,
        }
    }
}

impl Artifact {
    /// Number of objects in the underlying ObjectTable, if present.
    pub fn object_count(&self) -> Option<u32> {
        self.object_table()
            .map(|t| t.len() as u32)
    }

    /// Get a logical Object by object_id, if the ObjectTable is present
    /// and the id is within bounds.
    pub fn get_object(&self, id: u32) -> Option<Object> {
        let table = self.object_table()?;
        let idx = id as usize;
        let row = table.objects.get(idx)?;
        Some(Object::from_row(id, row))
    }
}
