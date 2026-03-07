# Quickstart

There are three ways to read a GeoTIFF with `cog3pio`'s Python bindings into CPU memory,
and two ways to read it into CUDA GPU memory.
Take your pick:

|  Output   | CUDA-acceleration | DLPack protocol | coordinates | any dtype |
|:---------:|:-----------------:|:---------------:|:-----------:|:---------:|
| PyCapsule |       ✅          |      ✅         |     ❌      |     ✅    |
| xarray    |       ✅          |      ❓         |     ✅      |     ✅    |
| numpy     |       ❌          |      ✅         |     ❌      |     ❌    |

Notes:

- [DLPack - an in-memory tensor structure](https://data-apis.org/array-api/latest/design_topics/data_interchange.html#dlpack-an-in-memory-tensor-structure)
- Coordinates in xarray are computed from the GeoTIFF's affine transformation
- Currently supported dtypes include:
    - On CPU: uint (u8/u16/u32/u64), int (i8/i16/i32/i64) and float (f16/f32/f64)
    - On CUDA: uint (u8/u16/u32/u64), int (i8/i16/i32/i64) and float (f32/f64)

## PyCapsule (DLPack)

Read a GeoTIFF file from a HTTP url via the [`CudaCogReader`][cog3pio.CudaCogReader] or
[`CogReader`][cog3pio.CogReader] class into an object that conforms to the
[Python Specification for DLPack](https://dmlc.github.io/dlpack/latest/python_spec.html),
whereby the `__dlpack__()` method returns a
[PyCapsule](https://docs.python.org/3/c-api/capsule.html#c.PyCapsule) object containing
a [`DLManagedTensorVersioned`](https://dmlc.github.io/dlpack/latest/c_api.html#c.DLManagedTensorVersioned)
object.

=== "CUDA"
    ```python
    import cupy as cp
    from cog3pio import CudaCogReader

    cog = CudaCogReader(
        path="https://github.com/OSGeo/gdal/raw/v3.11.0/autotest/gcore/data/float32.tif",
        device_id=0
    )
    assert hasattr(cog, "__dlpack__")
    assert hasattr(cog, "__dlpack_device__")

    array: cp.ndarray = cp.from_dlpack(cog)
    assert array.shape == (400,)  # (1, 20, 20)
    assert array.dtype == "float32"

    # or with Pytorch>=2.9.0, after https://github.com/pytorch/pytorch/pull/145000
    # tensor: torch.Tensor = torch.from_dlpack(cog)
    # ...
    ```
    
=== "CPU"
    ```python
    import numpy as np
    from cog3pio import CogReader

    cog = CogReader(
        path="https://github.com/OSGeo/gdal/raw/v3.11.0/autotest/gcore/data/float16.tif"
    )
    assert hasattr(cog, "__dlpack__")
    assert hasattr(cog, "__dlpack_device__")

    array: np.ndarray = np.from_dlpack(cog)
    assert array.shape == (1, 20, 20)
    assert array.dtype == "float16"

    # or with Pytorch>=2.9.0, after https://github.com/pytorch/pytorch/pull/145000
    # tensor: torch.Tensor = torch.from_dlpack(cog)
    # ...
    ```


## Xarray

Read GeoTIFF file from a HTTP url via the
[`Cog3pioBackendEntrypoint`](api#cog3pio.xarray_backend.Cog3pioBackendEntrypoint) engine
into an [xarray.DataArray][] object (akin to
[`rioxarray`](https://corteva.github.io/rioxarray)).
Set the
[`device`](api/#cog3pio.xarray_backend.Cog3pioBackendEntrypoint.open_dataset(device))
parameter to `(2, 0)` to read into CUDA (default), or `None` to read into CPU memory.

=== "CUDA"
    ```python
    import cupy as cp
    import xarray as xr

    # Read GeoTIFF into an xarray.DataArray
    dataarray: xr.DataArray = xr.open_dataarray(
        filename_or_obj="https://github.com/cogeotiff/rio-tiler/raw/7.8.0/tests/fixtures/cog_dateline.tif",
        engine="cog3pio",
        device=(2, 0),  # cuda:0
    )
    assert dataarray.sizes == {'band': 1, 'y': 2355, 'x': 2325}
    assert dataarray.dtype == "uint16"
    assert isinstance(dataarray.data, cp.ndarray)
    ```

=== "CPU"
    ```python
    import numpy as np
    import xarray as xr

    # Read GeoTIFF into an xarray.DataArray
    dataarray: xr.DataArray = xr.open_dataarray(
        filename_or_obj="https://github.com/cogeotiff/rio-tiler/raw/7.8.0/tests/fixtures/cog_dateline.tif",
        engine="cog3pio",
        device=None,  # or (1, 0) for cpu
    )
    assert dataarray.sizes == {'band': 1, 'y': 2355, 'x': 2325}
    assert dataarray.dtype == "uint16"
    assert isinstance(dataarray.data, np.ndarray)
    ```
    

## NumPy

Read a GeoTIFF file from a HTTP url via the [`read_geotiff`][cog3pio.read_geotiff]
function into a [`numpy.ndarray`][] (akin to
[`rasterio`](https://rasterio.readthedocs.io)).

```python
import numpy as np
from cog3pio import read_geotiff

# Read GeoTIFF into a numpy array
array: np.ndarray = read_geotiff(
    path="https://github.com/cogeotiff/rio-tiler/raw/6.4.0/tests/fixtures/cog_nodata_nan.tif"
)
assert array.shape == (1, 549, 549)  # bands, height, width
assert array.dtype == "float32"
```

!!! note

    The [`read_geotiff`][cog3pio.read_geotiff] function supports reading single or
    multi-band GeoTIFF files into a float32 array only. If you wish to read into other
    dtypes (e.g. uint16), please use the [Xarray](quickstart#xarray) or [DLPack](quickstart#pycapsule-dlpack) methods instead which supports reading into different
    dtypes.
