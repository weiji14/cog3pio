"""
Tests for xarray 'cog3pio' backend engine.
"""

import time

import cog3pio  # noqa: F401
import numpy as np
import pytest
import xarray as xr

try:
    import cupy as cp

    HAS_CUPY = True
except ImportError:
    HAS_CUPY = False


try:
    import rioxarray

    HAS_RIOXARRAY = True
except ImportError:
    HAS_RIOXARRAY = False


# %%
@pytest.mark.benchmark
@pytest.mark.parametrize(
    ("engine", "backend_kwargs"),
    [
        ("cog3pio", {"device": None}),  # CPU
        pytest.param(
            "cog3pio",
            {"device": (2, 0)},  # CUDA:0
            marks=pytest.mark.skipif(
                condition=not HAS_CUPY, reason="Could not import 'cupy'"
            ),
        ),
        pytest.param(
            "rasterio",  # CPU
            {},
            marks=pytest.mark.skipif(
                condition=not HAS_RIOXARRAY, reason="Could not import 'rioxarray'"
            ),
        ),
    ],
)
def test_xarray_backend_open_dataarray(engine, backend_kwargs):
    """
    Ensure that passing engine='cog3pio' to xarray.open_dataarray works, and benchmark
    against engine="rasterio" (rioxarray).
    """
    with xr.open_dataarray(
        filename_or_obj="https://github.com/cogeotiff/rio-tiler/raw/6.4.1/tests/fixtures/cog_nodata_nan.tif",
        engine=engine,
        backend_kwargs=backend_kwargs,
    ) as da:
        assert da.sizes == {'band': 1, 'y': 549, 'x': 549}
        assert da.x.min() == 500080.0
        assert da.x.max() == 609680.0
        assert da.y.min() == 5190340.0
        assert da.y.max() == 5299940.0
        assert da.dtype == "float32"
        # np.testing.assert_allclose(actual=da.mean(), desired=0.181176)
