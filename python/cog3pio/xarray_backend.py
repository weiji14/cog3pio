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
        reader = CogReader(path=filename_or_obj)

        array: np.ndarray = reader.to_numpy()
        x_coords, y_coords = reader.xy_coords()

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
