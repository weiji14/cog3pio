# Quickstart

There are a couple of ways to read a GeoTIFF with `cog3pio`'s Python bindings into
either CPU memory or CUDA GPU memory.

Take your pick:

|  Output   | CUDA-acceleration | DLPack protocol | any dtype |
|:---------:|:-----------------:|:---------------:|:---------:|
| PyCapsule |     ✅ / ❌       |      ✅         |     ✅    |
| numpy     |       ❌          |      ✅         |     ❌    |

Notes:

- [DLPack - an in-memory tensor structure](https://data-apis.org/array-api/latest/design_topics/data_interchange.html#dlpack-an-in-memory-tensor-structure)
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

    The [`read_geotiff`][cog3pio.read_geotiff] function is a convenience function that
    supports reading single or multi-band GeoTIFF files into a float32 array only. If
    you wish to read into other dtypes (e.g. uint16), please use the
    [DLPack](quickstart#pycapsule-dlpack) method instead which supports reading into
    different dtypes.
