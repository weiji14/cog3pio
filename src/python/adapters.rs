use std::io::Cursor;

use bytes::Bytes;
use dlpark::ffi::Device;
use dlpark::SafeManagedTensorVersioned;
use ndarray::Array3;
use numpy::{PyArray1, PyArray3, ToPyArray};
use object_store::{parse_url, ObjectStore};
use pyo3::exceptions::{PyBufferError, PyFileNotFoundError, PyValueError};
use pyo3::prelude::{pyclass, pyfunction, pymethods, pymodule, PyModule, PyResult, Python};
use pyo3::types::PyModuleMethods;
use pyo3::{wrap_pyfunction, Bound, PyErr};
use url::Url;

use crate::io::geotiff::{read_geotiff, CogReader};

/// Python class interface to a Cloud-optimized GeoTIFF reader.
///
/// Parameters
/// ----------
/// path : str
///     The path to the file, or a url to a remote file.
///
/// Returns
/// -------
/// reader : cog3pio.CogReader
///     A new CogReader instance for decoding GeoTIFF files.
///
/// Examples
/// --------
/// >>> import numpy as np
/// >>> from cog3pio import CogReader
/// >>>
/// >>> reader = CogReader(
/// >>>     path="https://github.com/rasterio/rasterio/raw/1.3.9/tests/data/float32.tif"
/// >>> )
/// >>> array: np.ndarray = reader.data()
/// >>> array.shape
/// >>> (1, 12, 13)
/// >>> array.dtype
/// >>> dtype('float32')
#[pyclass]
#[pyo3(name = "CogReader")]
struct PyCogReader {
    inner: CogReader<Cursor<Bytes>>,
}

#[pymethods]
impl PyCogReader {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let stream: Cursor<Bytes> = path_to_stream(path)?;
        let reader =
            CogReader::new(stream).map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(Self { inner: reader })
    }

    /// Get image pixel data from GeoTIFF as a DLPack capsule
    ///
    /// Returns
    /// -------
    /// tensor : PyCapsule
    ///     3D tensor of shape (band, height, width) containing the GeoTIFF pixel data.
    fn __dlpack__(&mut self) -> PyResult<SafeManagedTensorVersioned> {
        // Convert from ndarray (Rust) to DLPack (Python)
        let tensor: SafeManagedTensorVersioned = self
            .inner
            .dlpack()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(tensor)
    }

    /// Get device type and device ID in DLPack format.
    ///
    /// Meant for use by `from_dlpack()`.
    ///
    /// Returns
    /// -------
    /// device : (int, int)
    ///     A tuple (device_type, device_id) in DLPack format.
    #[staticmethod]
    fn __dlpack_device__() -> (i32, i32) {
        let device = Device::CPU;
        (device.device_type as i32, device.device_id)
    }

    /// Get x and y coordinates as numpy.ndarray
    #[allow(clippy::type_complexity)]
    fn xy_coords<'py>(
        &mut self,
        py: Python<'py>,
    ) -> PyResult<(Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>)> {
        let (x_coords, y_coords) = self
            .inner
            .xy_coords()
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok((x_coords.to_pyarray(py), y_coords.to_pyarray(py)))
    }
}

/// Read from a filepath or url into a byte stream
fn path_to_stream(path: &str) -> PyResult<Cursor<Bytes>> {
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
    Ok(stream)
}

/// Read a GeoTIFF file from a path on disk or a url into an ndarray
///
/// Parameters
/// ----------
/// path : str
///     The path to the file, or a url to a remote file.
///
/// Returns
/// -------
/// array : np.ndarray
///     3D array of shape (band, height, width) containing the GeoTIFF pixel data.
///
/// Examples
/// --------
/// from cog3pio import read_geotiff
///
/// array = read_geotiff("https://github.com/pka/georaster/raw/v0.1.0/data/tiff/float32.tif")
/// assert array.shape == (20, 20)
#[pyfunction]
#[pyo3(name = "read_geotiff")]
fn read_geotiff_py<'py>(path: &str, py: Python<'py>) -> PyResult<Bound<'py, PyArray3<f32>>> {
    // Open URL with TIFF decoder
    let stream = path_to_stream(path)?;

    // Decode TIFF into DLPack tensor
    let array: Array3<f32> =
        read_geotiff(stream).map_err(|err| PyValueError::new_err(err.to_string()))?;

    Ok(array.to_pyarray(py))
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn cog3pio(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register Python classes
    m.add_class::<PyCogReader>()?;
    // Register Python functions
    m.add_function(wrap_pyfunction!(read_geotiff_py, m)?)?;
    Ok(())
}
