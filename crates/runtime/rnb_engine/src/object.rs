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

    /// Execute a simple object-level query kernel against this artifact.
    ///
    /// The `arg` parameter is interpreted as:
    /// - `GetObjectById`: object_id
    /// - `ObjectsByType`: type_sid
    ///
    /// The result is always a list of logical `Object` views; for
    /// `GetObjectById` this is either empty or a single-element list.
    pub fn execute(
        &self,
        kernel: crate::QueryKernel,
        arg: u32,
    ) -> std::io::Result<Vec<Object>> {
        // If the manifest declares supported kernels, enforce that
        // the requested kernel is listed. For older artifacts that
        // do not populate `supported_kernels`, we skip this check.
        let manifest = self.manifest();
        if !manifest.supported_kernels.is_empty()
            && !manifest.supported_kernels.contains(&kernel)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "query kernel not supported by this artifact",
            ));
        }

        match kernel {
            crate::QueryKernel::GetObjectById => Ok(self.get_object(arg).into_iter().collect()),
            crate::QueryKernel::ObjectsByType => {
                let table = match self.object_table() {
                    Some(t) => t,
                    None => return Ok(Vec::new()),
                };

                let mut out = Vec::new();
                for (idx, row) in table.objects.iter().enumerate() {
                    if row.type_sid == arg {
                        out.push(Object::from_row(idx as u32, row));
                    }
                }
                Ok(out)
            }
            // Object-level execute should not be used for relation kernels.
            crate::QueryKernel::GetRelationsFrom | crate::QueryKernel::GetRelationsTo => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "relation kernels are not supported by Artifact::execute; use execute_relations",
                ))
            }
        }
    }

    /// Convenience wrapper for `ObjectsByType` kernels.
    pub fn objects_by_type(&self, type_sid: u32) -> std::io::Result<Vec<Object>> {
        self.execute(crate::QueryKernel::ObjectsByType, type_sid)
    }
}
