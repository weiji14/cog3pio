# cog3pio

Cloud-optimized GeoTIFF ... Parallel I/O

Yet another attempt at creating a GeoTIFF reader, in Rust, with Python bindings.

## Roadmap

Short term (Q1 2024):
- [ ] Implement single-band GeoTIFF reader to
      [`ndarray`](https://github.com/rust-ndarray/ndarray)
- [ ] Multi-band reader (relying on
      [`image-tiff`](https://github.com/image-rs/image-tiff))
- [ ] Read from remote storage (using
      [`object-store`](https://github.com/apache/arrow-rs/tree/object_store_0.9.0/object_store))

Medium term (Q2 2024):
- [ ] Integration with `xarray` as a
      [`BackendEntrypoint`](https://docs.xarray.dev/en/v2024.02.0/internals/how-to-add-new-backend.html)
- [ ] Parallel reader (TBD on multi-threaded or asynchronous)
- [ ] Direct-to-GPU loading

## Related crates

- https://github.com/georust/geotiff
- https://github.com/jblindsay/whitebox-tools
- https://github.com/pka/georaster
