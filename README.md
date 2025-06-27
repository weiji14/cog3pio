# cog3pio

Cloud-optimized GeoTIFF ... Parallel I/O

Yet another attempt at creating a GeoTIFF reader, in Rust, with Python bindings.


## Roadmap

Short term (Q1 2024):
- [x] Multi-band reader to [`ndarray`](https://github.com/rust-ndarray/ndarray) (relying
      on [`image-tiff`](https://github.com/image-rs/image-tiff))
- [x] Read from HTTP remote storage (using
      [`object-store`](https://github.com/apache/arrow-rs/tree/object_store_0.9.0/object_store))

Medium term (Q2-Q4 2024):
- [x] Integration with `xarray` as a
      [`BackendEntrypoint`](https://docs.xarray.dev/en/v2024.02.0/internals/how-to-add-new-backend.html)
- [x] Implement single-band GeoTIFF reader for multiple dtypes (uint/int/float) (based
      on [`geotiff`](https://github.com/georust/geotiff) crate)

Longer term (2025):
- [ ] Parallel reader (TBD on multi-threaded or asynchronous)
- [ ] Direct-to-GPU loading


## Related crates

- https://github.com/developmentseed/async-tiff
- https://github.com/feefladder/tiff2
- https://github.com/georust/geotiff
- https://github.com/jblindsay/whitebox-tools
- https://github.com/pka/georaster
