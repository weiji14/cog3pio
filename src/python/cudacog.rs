#![allow(clippy::doc_markdown)]
use std::io::Cursor;
use std::sync::Arc;

use bytes::Bytes;
use cudarc::driver::{CudaContext, CudaStream};
use dlpark::SafeManagedTensorVersioned;
use dlpark::ffi::{DLPACK_MAJOR_VERSION, DLPACK_MINOR_VERSION, Device};
use pyo3::exceptions::{PyBufferError, PyNotImplementedError, PyValueError, PyWarning};
use pyo3::{Bound, PyResult, Python, pyclass, pymethods};
use pyo3_stub_gen::define_stub_info_gatherer;
use pyo3_stub_gen_derive::{gen_stub_pyclass, gen_stub_pymethods};

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
/// device_id : int
///     The CUDA GPU device number to decode the TIFF data on.
///
/// Returns
/// -------
/// reader : cog3pio.CudaCogReader
///     A new CudaCogReader instance for decoding GeoTIFF files.
///
/// Raises
/// ------
/// ImportError
///     If `nvTIFF` is not installed. Please install it (e.g. via
///     `apt install nvtiff-cuda-13` or `dnf install nvtiff-cuda-13`) before using
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
/// ...     device_id=0,
/// ... )
/// >>> array: cp.ndarray = cp.from_dlpack(cog)
/// >>> array.shape
/// (2271752,)
/// >>> array.dtype
/// dtype('uint8')
#[gen_stub_pyclass]
#[pyclass(unsendable)]
#[pyo3(name = "CudaCogReader")]
pub(crate) struct PyCudaCogReader {
    inner: CudaCogReader,
    device: Device,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyCudaCogReader {
    #[new]
    #[pyo3(signature = (path, device_id))]
    fn new(path: &str, device_id: usize) -> PyResult<Self> {
        let stream: Cursor<Bytes> = path_to_stream(path)?;
        let bytes: Bytes = stream.into_inner();

        let cog =
            CudaCogReader::new(&bytes).map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(Self {
            inner: cog,
            device: Device::cuda(device_id),
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
    ///     - `None`: producer must assume the legacy default stream (default).
    ///     - `1`: the legacy default stream.
    ///     - `2`: the per-thread default stream.
    ///     - `> 2`: stream number represented as a Python integer.
    ///     - `0` is disallowed due to its ambiguity: `0` could mean either
    ///       `None`, `1`, or `2`.
    ///
    /// max_version : tuple[int, int] | None
    ///     The maximum DLPack version that the *consumer* (i.e., the caller of
    ///     `__dlpack__`) supports, in the form of a 2-tuple (`major`, `minor`). This
    ///     method may return a capsule of version `max_version` (recommended if it does
    ///     support that), or of a different version. This means the consumer must
    ///     verify the version even when `max_version` is passed.
    ///
    /// dl_device : tuple[int, int] | None
    ///     The DLPack device type. Default is `None`, meaning the exported capsule
    ///     should be on the same device as `self` is (i.e. CUDA). When specified, the
    ///     format must be a 2-tuple, following that of the return value of
    ///     [`array.__dlpack_device__()`][array_api.array.__dlpack_device__]. If the
    ///     device type cannot be handled by the producer, this function will raise
    ///     [BufferError][].
    ///
    /// copy : bool | None
    ///     Boolean indicating whether or not to copy the input. Currently only `None`
    ///     is supported, meaning the function must reuse the existing memory buffer if
    ///     possible and copy otherwise (copy is not actually implemented).
    ///
    /// Returns
    /// -------
    /// tensor : types.CapsuleType
    ///     1D tensor in row-major order containing the GeoTIFF pixel data.
    ///
    /// Raises
    /// ------
    /// NotImplementedError
    ///     If either of these cases happen:
    ///
    ///     - [`stream`][cog3pio.CudaCogReader.__dlpack__(stream)]>2 is passed in, as
    ///       only legacy default stream (1) or per-thread default stream (2) is
    ///       supported for now.
    ///     - [`max_version`](cog3pio.CudaCogReader.__dlpack__(max_version)) is
    ///       incompatible with the DLPack major version in this library.
    ///     - [`copy`](cog3pio.CudaCogReader.__dlpack__(copy)) is set to a value other
    ///       than `None` as
    ///       [Copy keyword argument behavior](https://data-apis.org/array-api/2025.12/design_topics/copies_views_and_mutation.html#copy-keyword-argument-behavior)
    ///       is not handled yet.
    /// BufferError
    ///     If trying to decode to non-CUDA memory, i.e. when
    ///     [`dl_device`][cog3pio.CudaCogReader.__dlpack__(dl_device)] is not `None`, or
    ///     set to a tuple other than `(2, x)`. This error may also be raised if trying
    ///     to decode to an unsupported version from the DLPack 0.x series.
    #[gen_stub(override_return_type(type_repr="types.CapsuleType", imports=("types")))]
    #[pyo3(signature = (stream=None, max_version=None, dl_device=None, copy=None))]
    fn __dlpack__(
        &self,
        stream: Option<u8>,
        max_version: Option<(u32, u32)>,
        dl_device: Option<(usize, usize)>,
        copy: Option<bool>,
    ) -> PyResult<SafeManagedTensorVersioned> {
        let device: Device = if let Some((device_type_int, device_id)) = dl_device {
            match device_type_int {
                2 => Ok(Device::cuda(device_id)),
                _ => Err(PyBufferError::new_err(format!(
                    "Only DLPack device_type 2 (CUDA) is allowed, got {device_type_int}"
                ))),
            }
        } else {
            Ok(self.device)
        }?;

        let ctx: Arc<CudaContext> = CudaContext::new(usize::try_from(device.device_id)?)
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

        if copy.is_some() {
            // TODO handle copy=True or copy=False
            dbg!(copy);
            Err(PyNotImplementedError::new_err(
                "`copy!=None` argument is not yet implemented.",
            ))
        } else {
            Ok(())
        }?;

        // Convert from ndarray (Rust) to DLPack (Python)
        let tensor: SafeManagedTensorVersioned = self
            .inner
            .dlpack(&cuda_stream)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;

        Ok(tensor)
    }

    /// Get device type and device ID in DLPack format.
    ///
    /// Meant for use by [`from_dlpack()`][array_api.from_dlpack].
    ///
    /// Returns
    /// -------
    /// device : (int, int)
    ///     A tuple (`device_type`, `device_id`) in DLPack format.
    fn __dlpack_device__(&self) -> (i32, i32) {
        (self.device.device_type as i32, self.device.device_id)
    }
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);
