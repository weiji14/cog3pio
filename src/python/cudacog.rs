#![allow(clippy::doc_markdown)]
use std::io::Cursor;
use std::sync::Arc;

use bytes::Bytes;
use cudarc::driver::{CudaContext, CudaStream};
use dlpark::SafeManagedTensorVersioned;
use dlpark::ffi::{DLPACK_MAJOR_VERSION, DLPACK_MINOR_VERSION, Device};
use pyo3::exceptions::{PyBufferError, PyNotImplementedError, PyValueError, PyWarning};
use pyo3::{Bound, PyAny, PyResult, pyclass, pymethods};

use crate::io::nvtiff::CudaCogReader;
use crate::python::adapters::path_to_stream;

/// Python class interface to a Cloud-optimized GeoTIFF reader (nvTIFF backend) that
/// decodes to GPU (CUDA) memory.
///
/// Warning
/// -------
/// This is an experimental feature only enabled on linux-x86_64 and linux-aarch64
/// wheel builds.
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
/// Raises
/// ------
/// ImportError
///     If ``nvTIFF`` is not installed. Please install it (e.g. via
///     ``apt install nvtiff-cuda-13`` or ``dnf install nvtiff-cuda-13``) before using
///     this class.
///
/// Examples
/// --------
/// Read a GeoTIFF from a HTTP url into a DLPack tensor:
///
/// >>> import cupy as cp
/// >>> from cog3pio import CudaCogReader
/// ...
/// >>> cog = CudaCogReader(
/// ...     path="https://github.com/rasterio/rasterio/raw/1.5.0/tests/data/RGBA.byte.tif"
/// ... )
/// >>> array: cp.ndarray = cp.from_dlpack(cog)
/// >>> array.shape
/// (2271752,)
/// >>> array.dtype
/// dtype('uint8')
#[pyclass(unsendable)]
#[pyo3(name = "CudaCogReader")]
pub(crate) struct PyCudaCogReader {
    inner: CudaCogReader,
    device: usize,
}

#[pymethods]
impl PyCudaCogReader {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let stream: Cursor<Bytes> = path_to_stream(path)?;
        let bytes: Bytes = stream.into_inner();

        let cog =
            CudaCogReader::new(&bytes).map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(Self {
            inner: cog,
            device: 0,
        })
    }

    /// Get image pixel data from GeoTIFF as a DLPack capsule.
    ///
    /// Parameters
    /// ----------
    /// stream : int | None
    ///     A Python integer representing a pointer to a stream, on devices that
    ///     support streams. Device-specific values of stream for CUDA:
    ///
    ///     - ``None``: producer must assume the legacy default stream (default).
    ///     - ``1``: the legacy default stream.
    ///     - ``2``: the per-thread default stream.
    ///     - ``> 2``: stream number represented as a Python integer.
    ///     - ``0`` is disallowed due to its ambiguity: ``0`` could mean either
    ///       ``None``, ``1``, or ``2``.
    ///
    /// max_version : tuple[int, int] | None
    ///     The maximum DLPack version that the consumer (i.e., the caller of
    ///     ``__dlpack__``) supports, in the form of a 2-tuple (``major``, ``minor``).
    ///     This method may return a capsule of version max_version (recommended if it
    ///     does support that), or of a different version. This means the consumer must
    ///     verify the version even when max_version is passed.
    ///
    /// Returns
    /// -------
    /// tensor : PyCapsule
    ///     1D tensor in row-major order containing the GeoTIFF pixel data.
    ///
    /// Raises
    /// ------
    /// NotImplementedError
    ///     If ``stream``>2 is passed in, as only legacy default stream (1) or
    ///     per-thread default stream (2) is supported for now. Or if ``max_version`` is
    ///     incompatible with the DLPack major version in this library.
    #[pyo3(signature = (stream=None, max_version=None, **kwargs))]
    fn __dlpack__(
        &self,
        stream: Option<u8>,
        max_version: Option<(u32, u32)>,
        kwargs: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<SafeManagedTensorVersioned> {
        // dbg!(stream, max_version, kwargs);

        let ctx: Arc<CudaContext> =
            CudaContext::new(self.device) // Set on GPU:0
                .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let cuda_stream: Arc<CudaStream> = if let Some(s_uint) = stream {
            match s_uint {
                0 => unreachable!(),              // disallowed due to ambiguity
                1 => Ok(ctx.default_stream()),    // legacy default stream
                2 => Ok(ctx.per_thread_stream()), // per-thread default stream
                3.. => Err(PyNotImplementedError::new_err(
                    "only legacy default stream (1) or per-thread default stream (2) is
                    supported for now, got {s_uint}",
                )),
            }
        } else {
            Ok(ctx.default_stream()) // None (default) means to assume legacy default stream
        }?;
        // dbg!(&cuda_stream);

        let _dlpack_version: PyResult<_> = if let Some((major, minor)) = max_version {
            if major >= DLPACK_MAJOR_VERSION && minor == DLPACK_MINOR_VERSION {
                Ok(())
            } else if major == DLPACK_MAJOR_VERSION {
                // accept minor version for now
                Err(PyWarning::new_err(format!(
                    "DLPack minor version mismatch: producer {DLPACK_MINOR_VERSION}, consumer {minor}. \
                    Using compatibility mode since major version ({DLPACK_MAJOR_VERSION}) is equal."
                )))
            } else {
                Err(PyNotImplementedError::new_err(
                    "Only supporting DLPack version {}.{}, but got {major}.{minor}",
                ))
            }
        } else {
            // no max_version given
            Err(PyBufferError::new_err("DLPack 0.X not supported"))
        };

        // Convert from ndarray (Rust) to DLPack (Python)
        let tensor: SafeManagedTensorVersioned = self
            .inner
            .dlpack(&cuda_stream)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(tensor)
    }

    /// Get device type and device ID in DLPack format.
    ///
    /// Meant for use by ``from_dlpack()``.
    ///
    /// Returns
    /// -------
    /// device : (int, int)
    ///     A tuple (``device_type``, ``device_id``) in DLPack format.
    fn __dlpack_device__(&self) -> (i32, i32) {
        let device = Device::cuda(self.device); // Hardcoded to GPU:0
        (device.device_type as i32, device.device_id)
    }
}
