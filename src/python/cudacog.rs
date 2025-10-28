#![allow(clippy::doc_markdown)]
use std::io::Cursor;
use std::sync::Arc;

use bytes::Bytes;
use cudarc::driver::{CudaContext, CudaStream};
use dlpark::SafeManagedTensorVersioned;
use dlpark::ffi::Device;
use pyo3::{Bound, PyAny, PyResult, pyclass, pymethods};

use crate::io::nvtiff::CudaCogReader;
use crate::python::adapters::path_to_stream;

/// Python class interface to a Cloud-optimized GeoTIFF reader (nvTIFF backend).
///
/// Parameters
/// ----------
/// path : str
///     The path to the file, or a url to a remote file.
///
/// Returns
/// -------
/// reader : cog3pio.CudaCogReader
///     A new CudaCogReader instance for decoding GeoTIFF files.
///
/// Examples
/// --------
/// Read a GeoTIFF from a HTTP url into a DLPack tensor:
///
/// >>> import cupy as cp
/// >>> from cog3pio import CudaCogReader
/// ...
/// >>> cog = CudaCogReader(
/// ...     path="https://github.com/rasterio/rasterio/raw/1.4.3/tests/data/RGBA.byte.tif"
/// ... )
/// >>> array: cp.ndarray = cp.from_dlpack(cog)
/// >>> array.shape
/// (2271752,)
/// >>> array.dtype
/// dtype('uint8')
#[pyclass]
#[pyo3(name = "CudaCogReader")]
pub(crate) struct PyCudaCogReader {
    inner: CudaCogReader,
}

#[pymethods]
impl PyCudaCogReader {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let stream: Cursor<Bytes> = path_to_stream(path)?;
        let bytes: Bytes = stream.into_inner();

        let ctx: Arc<CudaContext> = cudarc::driver::CudaContext::new(0).unwrap(); // Set on GPU:0
        let cuda_stream: Arc<CudaStream> = ctx.default_stream();

        let cog = CudaCogReader::new(&bytes, &cuda_stream).unwrap();
        //.map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(Self { inner: cog })
    }

    /// Get image pixel data from GeoTIFF as a DLPack capsule
    ///
    /// Returns
    /// -------
    /// tensor : PyCapsule
    ///     1D tensor in row-major order containing the GeoTIFF pixel data.
    #[pyo3(signature = (**kwargs))]
    fn __dlpack__(
        &self,
        kwargs: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<SafeManagedTensorVersioned> {
        dbg!(kwargs);

        // Convert from ndarray (Rust) to DLPack (Python)
        let tensor: SafeManagedTensorVersioned =
            self.inner.dlpack().expect("failed to convert to dlpack");
        // .map_err(|err| PyValueError::new_err(err.to_string()))?;

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
    fn __dlpack_device__(&self) -> (i32, i32) {
        let device = Device::cuda(self.inner.cuda_slice.context().ordinal());
        (device.device_type as i32, device.device_id)
    }
}
