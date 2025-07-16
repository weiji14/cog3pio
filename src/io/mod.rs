/// Read and write GeoTIFF files using the [`tiff`] crate (CPU backend)
pub mod geotiff;
/// Read and write GeoTIFF files using the [`nvtiff_sys`] crate (CUDA GPU backend)
#[cfg(feature = "cuda")]
pub mod nvtiff;
