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
//!
//! # Examples
//!
//! Read a GeoTIFF file retrieved via the [`object_store`] crate, and pass it into the
//! [`read_geotiff`](./io/geotiff/fn.read_geotiff.html) function.
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
//!         "https://github.com/cogeotiff/rio-tiler/raw/6.4.0/tests/fixtures/cog_nodata_nan.tif";
//!     let tif_url: Url = Url::parse(cog_url).unwrap();
//!     let (store, location): (Box<dyn ObjectStore>, Path) = parse_url(&tif_url).unwrap();
//!
//!     let stream: Cursor<Bytes> = {
//!         let result: GetResult = store.get(&location).await.unwrap();
//!         let bytes: Bytes = result.bytes().await.unwrap();
//!         Cursor::new(bytes)
//!     };
//!
//!     let arr: Array3<f32> = read_geotiff::<f32, _>(stream).unwrap();
//!     assert_eq!(arr.dim(), (1, 549, 549));
//!     assert_eq!(arr[[0, 500, 500]], 0.13482364);
//! }
//! ```
//!
//! Note that the output dtype can be specified either by using a type hint
//! (`let arr: Array3<f32>`) or via the turbofish operator (`read_geotiff::<f32>`).
//! Currently supported dtypes include uint (u8/u16/u32/u64), int (i8/i16/i32/i64) and
//! float (f32/f64).

/// Modules for handling Input/Output of GeoTIFF data
pub mod io;
/// Modules for Python to interface with Rust code
pub mod python;
