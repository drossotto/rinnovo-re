use crate::{Artifact, Object, StringDict};

/// High-level biological helpers over an `Artifact`.
///
/// This view assumes that the `StringDict` contains canonical type
/// labels such as `"cell"` and `"gene"`, and that object rows refer
/// to those labels via their `type_sid` field.
#[derive(Debug)]
pub struct BioView<'a> {
    artifact: &'a Artifact,
}

impl<'a> BioView<'a> {
    /// Construct a `BioView` if the artifact has both a string
    /// dictionary and an object table. Returns `None` otherwise.
    pub fn from_artifact(artifact: &'a Artifact) -> Option<Self> {
        if artifact.string_dict().is_some() && artifact.object_table().is_some() {
            Some(Self { artifact })
        } else {
            None
        }
    }

    fn string_dict(&self) -> &StringDict {
        // Safe by construction: `from_artifact` checked presence.
        self.artifact
            .string_dict()
            .expect("BioView requires a StringDict")
    }

    fn type_sid_for(&self, label: &str) -> std::io::Result<u32> {
        let dict = self.string_dict();
        if let Some(id) = dict
            .strings
            .iter()
            .position(|s| s == label)
            .map(|i| i as u32)
        {
            Ok(id)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("type label '{}' not found in StringDict", label),
            ))
        }
    }

    /// Return all objects whose type is the canonical `"cell"` label.
    pub fn cells(&self) -> std::io::Result<Vec<Object>> {
        let sid = self.type_sid_for("cell")?;
        self.artifact.objects_by_type(sid)
    }

    /// Return all objects whose type is the canonical `"gene"` label.
    pub fn genes(&self) -> std::io::Result<Vec<Object>> {
        let sid = self.type_sid_for("gene")?;
        self.artifact.objects_by_type(sid)
    }
}

