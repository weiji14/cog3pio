#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
//! # Cloud-optimized GeoTIFF ... Parallel I/O
//!
//! A reader for [Cloud Optimized GeoTIFF (COG)](https://www.cogeo.org) files.
//!
//! There are two backends. A CPU one and a GPU (CUDA) one.
//!
//! **Note**: Python bindings (via [`pyo3`]) are documented over at
//! <https://cog3pio.readthedocs.io>.
//!
//! # CPU decoder
//!
//! Uses [`tiff`] to decode TIFF images. Pixel data is stored with a shape of
//! (channels, height, width) using either:
//! - [`CogReader`](crate::io::geotiff::CogReader) - returns a
//!   [DLPack](https://dmlc.github.io/dlpack/latest) data structure in CPU-memory via
//!   [`dlpark`](https://docs.rs/dlpark)
//! - [`read_geotiff`](crate::io::geotiff::read_geotiff) - returns a 3D Array via
//!   [`ndarray`](https://docs.rs/ndarray)
//!
//! ## Examples
//!
//! ### DLPack
//!
//! Retrieve a GeoTIFF file stream via the [`object_store`] crate, pass it into the
//! [`CogReader::new`](crate::io::geotiff::CogReader::new) method to instantiate a
//! [`CogReader`](crate::io::geotiff::CogReader) struct, and call
//! [`.dlpack()`](crate::io::geotiff::CogReader::dlpack) to get a
//! [`dlpark::SafeManagedTensorVersioned`] output.
//!
//! ```rust
//! use std::io::Cursor;
//!
//! use bytes::Bytes;
//! use cog3pio::io::geotiff::CogReader;
//! use dlpark::ffi::DataType;
//! use dlpark::prelude::TensorView;
//! use dlpark::SafeManagedTensorVersioned;
//! use object_store::path::Path;
//! use object_store::{parse_url, GetResult, ObjectStore};
//! use tokio;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cog_url: &str =
//!         "https://github.com/cogeotiff/rio-tiler/raw/7.8.0/tests/fixtures/cog_dateline.tif";
//!     let tif_url: Url = Url::parse(cog_url).unwrap();
//!     let (store, location): (Box<dyn ObjectStore>, Path) = parse_url(&tif_url).unwrap();
//!
//!     let stream: Cursor<Bytes> = {
//!         let result: GetResult = store.get(&location).await.unwrap();
//!         let bytes: Bytes = result.bytes().await.unwrap();
//!         Cursor::new(bytes)
//!     };
//!
//!     // Read GeoTIFF into a dlpark::versioned::SafeManagedTensorVersioned
//!     let mut cog = CogReader::new(stream).unwrap();
//!     let tensor: SafeManagedTensorVersioned = cog.dlpack().unwrap();
//!     assert_eq!(tensor.shape(), [1, 2355, 2325]);
//!     assert_eq!(tensor.data_type(), &DataType::U16);
//! }
//! ```
//!
//! ### Ndarray
//!
//! Retrieve a GeoTIFF file stream via the [`object_store`] crate, and pass it into the
//! [`read_geotiff`](crate::io::geotiff::read_geotiff) function to get an
//! [`ndarray::Array3`] output.
//!
//! ```rust
//! use std::io::Cursor;
//!
//! use bytes::Bytes;
//! use cog3pio::io::geotiff::read_geotiff;
//! use ndarray::Array3;
//! use object_store::path::Path;
//! use object_store::{parse_url, GetResult, ObjectStore};
//! use tokio;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cog_url: &str =
//!         "https://github.com/cogeotiff/rio-tiler/raw/7.8.0/tests/fixtures/cog_nodata_nan.tif";
//!     let tif_url: Url = Url::parse(cog_url).unwrap();
//!     let (store, location): (Box<dyn ObjectStore>, Path) = parse_url(&tif_url).unwrap();
//!
//!     let stream: Cursor<Bytes> = {
//!         let result: GetResult = store.get(&location).await.unwrap();
//!         let bytes: Bytes = result.bytes().await.unwrap();
//!         Cursor::new(bytes)
//!     };
//!
//!     // Read GeoTIFF into an ndarray::Array
//!     let arr: Array3<f32> = read_geotiff::<f32, _>(stream).unwrap();
//!     assert_eq!(arr.dim(), (1, 549, 549));
//!     assert_eq!(arr[[0, 500, 500]], 0.13482364);
//! }
//! ```
//!
//! Note that the output dtype is specified either using a type hint
//! (`let arr: Array3<f32>`) or via a turbofish operator (`read_geotiff::<f32, _>`).
//! Currently supported dtypes include uint (u8/u16/u32/u64), int (i8/i16/i32/i64) and
//! float (f16/f32/f64).
//!
//! # GPU (CUDA) decoder
//!
//! Uses [`nvtiff_sys`] to decode TIFF images. Pixel data is stored as a flattened 1D
//! array in row-major order (i.e. rows-first, columns-next). Use:
//! - [`CudaCogReader`](crate::io::nvtiff::CudaCogReader) - returns a
//!   [DLPack](https://dmlc.github.io/dlpack/latest) data structure in CUDA-memory via
//!   [`dlpark`](https://docs.rs/dlpark)

/// Modules for handling Input/Output of GeoTIFF data
pub mod io;
/// Modules for Python to interface with Rust code
#[cfg(feature = "pyo3")]
mod python;
