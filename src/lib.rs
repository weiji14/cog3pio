#![warn(missing_docs)]
//! # Cloud-optimized GeoTIFF ... Parallel I/O
//!
//! A reader for [Cloud Optimized GeoTIFF (COG)](https://www.cogeo.org) files.
//!
//! Uses [`tiff`] to decode TIFF images, storing the pixel data in [`ndarray`] structs.
//!
//! **Note**: For Python users, there are also bindings (via [`pyo3`]) to read GeoTIFF files into
//! `numpy.ndarray` objects (i.e. similar to [`rasterio`](https://github.com/rasterio/rasterio)).
//! This is done via the [`numpy`] crate which enables passing data from Rust to Python.

/// Modules for handling Input/Output of GeoTIFF data
pub mod io;

use std::fs::File;

use ndarray::Dim;
use numpy::{PyArray, ToPyArray};
use pyo3::prelude::{pyfunction, pymodule, PyModule, PyResult, Python};
use pyo3::wrap_pyfunction;

/// Read a GeoTIFF file from a path on disk into an ndarray
///
/// Parameters
/// ----------
/// path : str
///     The path to the file.
///
/// Returns
/// -------
/// array : np.ndarray
///     2D array containing the GeoTIFF pixel data.
#[pyfunction]
#[pyo3(name = "read_geotiff")]
fn read_geotiff_py<'py>(
    path: &str,
    py: Python<'py>,
) -> PyResult<&'py PyArray<f32, Dim<[usize; 2]>>> {
    // Open TIFF file from path
    let file = File::open(path).expect("Cannot find GeoTIFF file");
    // Get image pixel data as an ndarray
    let vec_data = io::geotiff::read_geotiff(file).expect("Cannot read GeoTIFF");
    // Convert from ndarray (Rust) to numpy ndarray (Python)
    Ok(vec_data.to_pyarray(py))
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn cog3pio(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_geotiff_py, m)?)?;
    Ok(())
}
