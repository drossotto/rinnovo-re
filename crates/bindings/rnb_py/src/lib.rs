use pyo3::prelude::*;
use pyo3::types::PyBytes;

use rnb_engine;

// Simple hello to verify extension is importable.
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
pub struct RnbFilePy {
    #[pyo3(get)]
    pub header: Header,
    #[pyo3(get)]
    pub manifest: Manifest,
}

impl From<rnb_engine::RnbFile> for RnbFilePy {
    fn from(f: rnb_engine::RnbFile) -> Self {
        Self {
            header: Header::from(f.header),
            manifest: Manifest::from(f.manifest),
        }
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
    // Ensure the error is surfaced as IOError to Python.
    let f = rnb_engine::open(path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
    // Optionally, we could expose directory or raw bytes later; for now
    // we return just header + manifest to keep the surface minimal.
    Ok(RnbFilePy::from(f))
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
    m.add_function(wrap_pyfunction!(read_manifest_bytes, m)?)?;

    // Provide numeric constants for segment and kernel identifiers so
    // Python callers do not have to hard-code magic numbers.
    m.add("SEGMENT_MANIFEST", rnb_engine::SegmentType::Manifest.as_u32())?;
    m.add("SEGMENT_STRING_DICT", rnb_engine::SegmentType::StringDict.as_u32())?;
    m.add("SEGMENT_OBJECT_TABLE", rnb_engine::SegmentType::ObjectTable.as_u32())?;

    m.add("KERNEL_GET_OBJECT_BY_ID", rnb_engine::QueryKernel::GetObjectById.as_u32())?;

    m.add_class::<Header>()?;
    m.add_class::<Manifest>()?;
    m.add_class::<RnbFilePy>()?;

    Ok(())
}
