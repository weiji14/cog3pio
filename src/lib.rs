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

use std::io::Cursor;

use bytes::Bytes;
use ndarray::Dim;
use numpy::{PyArray, ToPyArray};
use object_store::{parse_url, ObjectStore};
use pyo3::exceptions::{PyBufferError, PyFileNotFoundError, PyValueError};
use pyo3::prelude::{pyfunction, pymodule, PyModule, PyResult, Python};
use pyo3::{wrap_pyfunction, PyErr};
use url::Url;

/// Read a GeoTIFF file from a path on disk into an ndarray
///
/// Parameters
/// ----------
/// path : str
///     The path to the file, or a url to a remote file.
///
/// Returns
/// -------
/// array : np.ndarray
///     2D array containing the GeoTIFF pixel data.
///
/// Examples
/// --------
/// from cog3pio import read_geotiff
///
/// array = read_geotiff("https://github.com/pka/georaster/raw/v0.1.0/data/tiff/float32.tif")
/// assert array.shape == (20, 20)
#[pyfunction]
#[pyo3(name = "read_geotiff")]
fn read_geotiff_py<'py>(
    path: &str,
    py: Python<'py>,
) -> PyResult<&'py PyArray<f32, Dim<[usize; 2]>>> {
    // Parse URL into ObjectStore and path
    let file_or_url = match Url::from_file_path(path) {
        // Parse local filepath
        Ok(filepath) => filepath,
        // Parse remote URL
        Err(_) => Url::parse(path)
            .map_err(|_| PyValueError::new_err(format!("Cannot parse path: {path}")))?,
    };
    let (store, location) = parse_url(&file_or_url)
        .map_err(|_| PyValueError::new_err(format!("Cannot parse url: {file_or_url}")))?;

    // Initialize async runtime
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Get TIFF file stream asynchronously
    let stream = runtime.block_on(async {
        let result = store
            .get(&location)
            .await
            .map_err(|_| PyFileNotFoundError::new_err(format!("Cannot find file: {path}")))?;
        let bytes = result.bytes().await.map_err(|_| {
            PyBufferError::new_err(format!("Failed to stream data from {path} into bytes."))
        })?;
        // Return cursor to in-memory buffer
        Ok::<Cursor<Bytes>, PyErr>(Cursor::new(bytes))
    })?;

    // Get image pixel data as an ndarray
    let vec_data = io::geotiff::read_geotiff(stream)
        .map_err(|err| PyValueError::new_err(format!("Cannot read GeoTIFF because: {err}")))?;

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
