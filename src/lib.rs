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

use ndarray::Dim;
use numpy::{PyArray, ToPyArray};
use object_store::{parse_url, ObjectStore};
use pyo3::prelude::{pyfunction, pymodule, PyModule, PyResult, Python};
use pyo3::wrap_pyfunction;
use tokio;
use url::ParseError;
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
    // Parse URL, prepend file:// for local filepaths
    let url = match Url::parse(path) {
        Ok(url) => url,
        Err(ParseError::RelativeUrlWithoutBase) => {
            let new_path = "file://".to_owned() + path;
            let url = Url::parse(new_path.as_str()).expect(&format!("Cannot parse path: {path}"));
            url
        }
        Err(e) => Err(format!("{}", e)).unwrap(),
    };
    let (store, location) = parse_url(&url).expect(&format!("Cannot parse url: {url}"));

    // Initialize async runtime
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Get TIFF file stream asynchronously
    let stream = runtime.block_on(async {
        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        // Return cursor to in-memory buffer
        Cursor::new(bytes)
    });

    // Get image pixel data as an ndarray
    let vec_data = io::geotiff::read_geotiff(stream).expect("Cannot read GeoTIFF");
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
