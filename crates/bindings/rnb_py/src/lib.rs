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
    #[pyo3(get)]
    pub header: Header,
    #[pyo3(get)]
    pub manifest: Manifest,
}

impl From<rnb_engine::Artifact> for RnbFilePy {
    fn from(a: rnb_engine::Artifact) -> Self {
        let header = Header::from(*a.header());
        let manifest = Manifest::from(a.manifest().clone());
        Self { header, manifest }
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
    let objs = art
        .execute(rnb_engine::QueryKernel::GetObjectById, object_id)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(objs.into_iter().next().map(Object::from))
}

/// Execute the `ObjectsByType` kernel and return all matching Objects.
#[pyfunction]
fn objects_by_type(path: &str, type_sid: u32) -> PyResult<Vec<Object>> {
    let art = rnb_engine::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    let objs = art
        .objects_by_type(type_sid)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    Ok(objs.into_iter().map(Object::from).collect())
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
