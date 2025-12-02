# API Reference

These are the Python docs for `cog3pio`, for the Rust docs, see
<https://docs.rs/cog3pio>.

## DLPack

```{eval-rst}
.. autoclass:: cog3pio.CogReader
    :members:
    :special-members: __dlpack__, __dlpack_device__
.. autoclass:: cog3pio.CudaCogReader
    :members:
    :special-members: __dlpack__, __dlpack_device__
```

## Xarray

```{eval-rst}
.. autoclass:: cog3pio.xarray_backend.Cog3pioBackendEntrypoint
```


## NumPy

```{eval-rst}
.. autofunction:: cog3pio.read_geotiff
```
