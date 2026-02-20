use std::path::Path;

use crate::{
    Manifest,
    ObjectTable,
    RnbDirectory,
    RnbFile,
    RnbHeader,
    StringDict,
};

/// High-level wrapper around a parsed RNB artifact.
///
/// This type exposes a stable, semantic view over the underlying
/// `rnb_format::RnbFile` while hiding the low-level I/O details.
#[derive(Debug, Clone)]
pub struct Artifact {
    inner: RnbFile,
}

impl Artifact {
    /// Open an RNB artifact from the given path.
    ///
    /// This method validates the header, directory, manifest, and any
    /// invariants enforced by the container (e.g. required segments).
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = rnb_format::open_rnb(path)?;
        Ok(Self { inner: file })
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
}

