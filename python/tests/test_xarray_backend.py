"""
Tests for xarray 'cog3pio' backend engine.
"""

import xarray as xr


# %%
def test_xarray_backend_cog3pio():
    """
    Ensure that passing engine='cog3pio' to xarray.open_dataarray works.
    """
    with xr.open_dataarray(
        filename_or_obj="https://github.com/cogeotiff/rio-tiler/raw/6.4.1/tests/fixtures/cog_nodata_nan.tif",
        engine="cog3pio",
    ) as da:
        assert da.sizes == {'band': 1, 'y': 549, 'x': 549}
        assert da.dtype == "float32"
