/// Read and write GeoTIFF files using the `image-tiff` crate
pub mod geotiff;
/// Read and write GeoTIFF files using the `nvtiff-sys` crate
#[cfg(feature = "cuda")]
pub mod nvtiff;
