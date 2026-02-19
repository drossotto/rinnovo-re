use pyo3::prelude::*;

#[pyfunction]
fn hello() -> String {
    "rinnovo ready".to_string()
}

#[pymodule]
fn rinnovo(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    Ok(())
}