use pyo3::prelude::*;
use pyo3::types::PyBytes;

use rnb_engine;

#[pyfunction]
fn hello() -> String {
    "rinnovo ready".to_string()
}

#[pyclass]
#[derive(Clone)]
pub struct Header {
    #[pyo3(get)]
    pub magic: Vec<u8>,
    #[pyo3(get)]
    pub version_major: u16,
    #[pyo3(get)]
    pub version_minor: u16,
    #[pyo3(get)]
    pub dir_offset: u64,
    #[pyo3(get)]
    pub dir_len: u64,
}

impl From<rnb_engine::RnbHeader> for Header {
    fn from(h: rnb_engine::RnbHeader) -> Self {
        Self {
            magic: h.magic.to_vec(),
            version_major: h.version_major,
            version_minor: h.version_minor,
            dir_offset: h.dir_offset,
            dir_len: h.dir_len,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Manifest {
    #[pyo3(get)]
    pub flags: u32,
    #[pyo3(get)]
    pub required_segments: Vec<u32>,
    #[pyo3(get)]
    pub supported_kernels: Vec<u32>,
    #[pyo3(get)]
    pub max_chunk_bytes: u32,
}

impl From<rnb_engine::Manifest> for Manifest {
    fn from(m: rnb_engine::Manifest) -> Self {
        Self {
            flags: m.flags,
            required_segments: m.required_segments.iter().map(|s| s.as_u32()).collect(),
            supported_kernels: m.supported_kernels.iter().map(|k| k.as_u32()).collect(),
            max_chunk_bytes: m.max_chunk_bytes,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Object {
    #[pyo3(get)]
    pub id: u32,
    #[pyo3(get)]
    pub type_sid: u32,
    #[pyo3(get)]
    pub name_sid: u32,
    #[pyo3(get)]
    pub flags: u32,
}

impl From<rnb_engine::Object> for Object {
    fn from(o: rnb_engine::Object) -> Self {
        Self {
            id: o.id,
            type_sid: o.type_sid,
            name_sid: o.name_sid,
            flags: o.flags,
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct RnbFilePy {
    // Full in-memory artifact; not exposed directly to Python.
    inner: rnb_engine::Artifact,
    #[pyo3(get)]
    pub header: Header,
    #[pyo3(get)]
    pub manifest: Manifest,
}

impl From<rnb_engine::Artifact> for RnbFilePy {
    fn from(a: rnb_engine::Artifact) -> Self {
        let header = Header::from(*a.header());
        let manifest = Manifest::from(a.manifest().clone());
        Self {
            inner: a,
            header,
            manifest,
        }
    }
}

#[pymethods]
impl RnbFilePy {
    /// Return the underlying StringDict as a list of strings.
    ///
    /// If the artifact has no StringDict segment, returns an empty list.
    pub fn list_strings(&self) -> Vec<String> {
        match self.inner.string_dict() {
            Some(d) => d.strings.clone(),
            None => Vec::new(),
        }
    }

    /// Execute the `GetObjectById` kernel and return at most one Object.
    pub fn get_object(&self, object_id: u32) -> PyResult<Option<Object>> {
        let objs = self
            .inner
            .execute(rnb_engine::QueryKernel::GetObjectById, object_id)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(objs.into_iter().next().map(Object::from))
    }

    /// Execute the `ObjectsByType` kernel and return all matching Objects.
    pub fn objects_by_type(&self, type_sid: u32) -> PyResult<Vec<Object>> {
        let objs = self
            .inner
            .objects_by_type(type_sid)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        Ok(objs.into_iter().map(Object::from).collect())
    }

    /// List all attribute records for a given `object_id`.
    ///
    /// The return value is a list of `(object_id, key_sid, value_sid, flags)` tuples.
    pub fn list_attributes(&self, object_id: u32) -> PyResult<Vec<(u32, u32, u32, u32)>> {
        let iter = match self.inner.attributes_for_object(object_id) {
            Some(it) => it,
            None => return Ok(Vec::new()),
        };

        Ok(iter
            .map(|a| (a.object_id, a.key_sid, a.value_sid, a.flags))
            .collect())
    }

    /// List relations filtered by optional src, dst, and relation type.
    ///
    /// If both `src_id` and `dst_id` are `None`, all relations in the table are
    /// returned (optionally filtered by `rel_type_sid`).
    pub fn list_relations(
        &self,
        src_id: Option<u32>,
        dst_id: Option<u32>,
        rel_type_sid: Option<u32>,
    ) -> PyResult<Vec<(u32, u32, u32, u32)>> {
        let table = match self.inner.relation_table() {
            Some(t) => t,
            None => return Ok(Vec::new()),
        };

        let rels = table.relations.iter().filter(|r| {
            if let Some(s) = src_id {
                if r.src_id != s {
                    return false;
                }
            }
            if let Some(d) = dst_id {
                if r.dst_id != d {
                    return false;
                }
            }
            if let Some(t) = rel_type_sid {
                if r.rel_type_sid != t {
                    return false;
                }
            }
            true
        });

        Ok(rels
            .map(|r| (r.src_id, r.dst_id, r.rel_type_sid, r.flags))
            .collect())
    }
}

/// Write a minimal RNB artifact to the given path.
#[pyfunction]
fn write_empty(path: &str) -> PyResult<()> {
    rnb_engine::write_empty(path).map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
}

/// Open an RNB artifact and return a lightweight Python view.
#[pyfunction]
fn open(_py: Python<'_>, path: &str) -> PyResult<RnbFilePy> {
    let art = rnb_engine::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(RnbFilePy::from(art))
}

/// Execute the `GetObjectById` kernel and return at most one Object.
#[pyfunction]
fn get_object(path: &str, object_id: u32) -> PyResult<Option<Object>> {
    let art = rnb_engine::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let wrapper = RnbFilePy::from(art);
    wrapper.get_object(object_id)
}

/// Execute the `ObjectsByType` kernel and return all matching Objects.
#[pyfunction]
fn objects_by_type(path: &str, type_sid: u32) -> PyResult<Vec<Object>> {
    let art = rnb_engine::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let wrapper = RnbFilePy::from(art);
    wrapper.objects_by_type(type_sid)
}

/// List all attribute records for a given `object_id`.
///
/// The return value is a list of `(object_id, key_sid, value_sid, flags)` tuples.
#[pyfunction]
fn list_attributes(path: &str, object_id: u32) -> PyResult<Vec<(u32, u32, u32, u32)>> {
    let art = rnb_engine::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let wrapper = RnbFilePy::from(art);
    wrapper.list_attributes(object_id)
}

/// List relations filtered by optional src, dst, and relation type.
///
/// If both `src_id` and `dst_id` are `None`, all relations in the table are
/// returned (optionally filtered by `rel_type_sid`).
#[pyfunction]
fn list_relations(
    path: &str,
    src_id: Option<u32>,
    dst_id: Option<u32>,
    rel_type_sid: Option<u32>,
) -> PyResult<Vec<(u32, u32, u32, u32)>> {
    let art = rnb_engine::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let wrapper = RnbFilePy::from(art);
    wrapper.list_relations(src_id, dst_id, rel_type_sid)
}

/// Return the underlying StringDict as a list of strings.
///
/// If the artifact has no StringDict segment, an empty list is returned.
#[pyfunction]
fn list_strings(path: &str) -> PyResult<Vec<String>> {
    let art = rnb_engine::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let wrapper = RnbFilePy::from(art);
    Ok(wrapper.list_strings())
}

/// Convenience: return the raw manifest bytes for an artifact.
#[pyfunction]
fn read_manifest_bytes(py: Python<'_>, path: &str) -> PyResult<PyObject> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    let mut f = File::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let header = rnb_engine::RnbHeader::read_from(&mut f)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    f.seek(SeekFrom::Start(header.dir_offset))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let dir = rnb_engine::RnbDirectory::read_from(&mut f, header.dir_len)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let manifest_entry = dir
        .entries
        .iter()
        .find(|e| e.segment_type == rnb_engine::SegmentType::Manifest.as_u32())
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("missing manifest segment"))?;

    f.seek(SeekFrom::Start(manifest_entry.offset))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    let mut buf = vec![0u8; manifest_entry.length as usize];
    f.read_exact(&mut buf)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;

    Ok(PyBytes::new(py, &buf).into())
}

#[pymodule]
fn rinnovo(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    m.add_function(wrap_pyfunction!(write_empty, m)?)?;
    m.add_function(wrap_pyfunction!(open, m)?)?;
    m.add_function(wrap_pyfunction!(get_object, m)?)?;
    m.add_function(wrap_pyfunction!(objects_by_type, m)?)?;
    m.add_function(wrap_pyfunction!(list_attributes, m)?)?;
    m.add_function(wrap_pyfunction!(list_relations, m)?)?;
    m.add_function(wrap_pyfunction!(list_strings, m)?)?;
    m.add_function(wrap_pyfunction!(read_manifest_bytes, m)?)?;

    // Provide numeric constants for segment and kernel identifiers so
    // Python callers do not have to hard-code magic numbers.
    m.add("SEGMENT_MANIFEST", rnb_engine::SegmentType::Manifest.as_u32())?;
    m.add("SEGMENT_STRING_DICT", rnb_engine::SegmentType::StringDict.as_u32())?;
    m.add("SEGMENT_OBJECT_TABLE", rnb_engine::SegmentType::ObjectTable.as_u32())?;

    m.add("KERNEL_GET_OBJECT_BY_ID", rnb_engine::QueryKernel::GetObjectById.as_u32())?;
    m.add("KERNEL_OBJECTS_BY_TYPE", rnb_engine::QueryKernel::ObjectsByType.as_u32())?;

    m.add_class::<Header>()?;
    m.add_class::<Manifest>()?;
    m.add_class::<RnbFilePy>()?;
    m.add_class::<Object>()?;

    Ok(())
}
