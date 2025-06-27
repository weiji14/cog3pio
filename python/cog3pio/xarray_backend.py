"""
An xarray backend for reading GeoTIFF files using the 'cog3pio' engine.
"""

import os

import numpy as np
import xarray as xr
from xarray.backends import BackendEntrypoint

from cog3pio import CogReader


# %%
class Cog3pioBackendEntrypoint(BackendEntrypoint):
    """
    Xarray backend to read GeoTIFF files using 'cog3pio' engine.

    Examples
    --------
    Read a GeoTIFF from a HTTP url into an xarray.DataArray:

    >>> import xarray as xr
    ...
    >>> # Read GeoTIFF into an xarray.DataArray
    >>> dataarray: xr.DataArray = xr.open_dataarray(
    ...     filename_or_obj="https://github.com/OSGeo/gdal/raw/v3.11.0/autotest/gcore/data/byte_zstd.tif",
    ...     engine="cog3pio",
    ... )
    >>> dataarray.sizes
    Frozen({'band': 1, 'y': 20, 'x': 20})
    >>> dataarray.dtype
    dtype('uint8')
    """

    description = "Use .tif files in Xarray"
    open_dataset_parameters = ["filename_or_obj"]
    url = "https://github.com/weiji14/cog3pio"

    def open_dataset(
        self,
        filename_or_obj: str,
        *,
        drop_variables=None,
        # other backend specific keyword arguments
        # `chunks` and `cache` DO NOT go here, they are handled by xarray
    ) -> xr.Dataset:
        cog = CogReader(path=filename_or_obj)

        array: np.ndarray = np.from_dlpack(cog)
        x_coords, y_coords = cog.xy_coords()

        channels, height, width = array.shape
        dataarray: xr.DataArray = xr.DataArray(
            data=array,
            coords={
                "band": np.arange(stop=channels, dtype=np.uint8),
                "y": y_coords,
                "x": x_coords,
            },
            name=None,
            attrs=None,
        )

        return dataarray.to_dataset(name="raster")

    def guess_can_open(self, filename_or_obj):
        try:
            _, ext = os.path.splitext(filename_or_obj)
        except TypeError:
            return False
        return ext in {".tif", ".tiff"}
