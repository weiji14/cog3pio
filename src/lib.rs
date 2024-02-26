use pyo3::prelude::{pymodule, PyModule, PyResult, Python};

/// A Python module implemented in Rust.
#[pymodule]
fn cog3pio(_py: Python, m: &PyModule) -> PyResult<()> {
    Ok(())
}
