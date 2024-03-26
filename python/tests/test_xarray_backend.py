"""
Tests for xarray 'cog3pio' backend engine.
"""

import numpy as np
import pytest
import xarray as xr

try:
    import rioxarray

    HAS_RIOXARRAY = True
except ImportError:
    HAS_RIOXARRAY = False


# %%
@pytest.mark.benchmark
@pytest.mark.parametrize(
    "engine",
    [
        "cog3pio",
        pytest.param(
            "rasterio",
            marks=pytest.mark.skipif(
                condition=not HAS_RIOXARRAY, reason="Could not import 'rioxarray'"
            ),
        ),
    ],
)
def test_xarray_backend_open_dataarray(engine):
    """
    Ensure that passing engine='cog3pio' to xarray.open_dataarray works, and benchmark
    against engine="rasterio" (rioxarray).
    """
    with xr.open_dataarray(
        filename_or_obj="https://github.com/cogeotiff/rio-tiler/raw/6.4.1/tests/fixtures/cog_nodata_nan.tif",
        engine=engine,
    ) as da:
        assert da.sizes == {'band': 1, 'y': 549, 'x': 549}
        assert da.x.min() == 499980.0
        assert da.x.max() == 609580.0
        assert da.y.min() == 5190440.0
        assert da.y.max() == 5300040.0
        assert da.dtype == "float32"
        np.testing.assert_allclose(actual=da.mean(), desired=0.08855476)
