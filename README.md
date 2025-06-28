# cog3pio

Cloud-optimized GeoTIFF ... Parallel I/O

Yet another attempt at creating a GeoTIFF reader, in Rust, with Python bindings.


## Roadmap

2024 Q1:
- [x] Multi-band reader to [`ndarray`](https://github.com/rust-ndarray/ndarray) (relying
      on [`image-tiff`](https://crates.io/crates/tiff))
- [x] Read from HTTP remote storage (using
      [`object-store`](https://crates.io/crates/object_store))

2024 Q2-Q4:
- [x] Integration with `xarray` as a
      [`BackendEntrypoint`](https://docs.xarray.dev/en/v2024.02.0/internals/how-to-add-new-backend.html)
- [x] Implement single-band GeoTIFF reader for multiple dtypes (uint/int/float) (based
      on [`geotiff`](https://crates.io/crates/geotiff) crate)

2025 Q1-Q2:
- [x] Support for [`DLPack`](https://dmlc.github.io/dlpack/latest/index.html) protocol
      (through [`dlpark`](https://crates.io/crates/dlpark))
- [x] Initial release on crates.io and PyPI

2025 Q3-Q4:
- [ ] GPU-based decoding (via [`nvTIFF`](https://crates.io/crates/nvtiff-sys))
- [ ] Asynchronous I/O (refactor to [`async-tiff`](https://crates.io/crates/async-tiff))

2026:
- [ ] Direct-to-GPU loading


## Related crates

- https://github.com/developmentseed/async-tiff
- https://github.com/feefladder/tiff2
- https://github.com/georust/geotiff
- https://github.com/jblindsay/whitebox-tools
- https://github.com/pka/georaster
