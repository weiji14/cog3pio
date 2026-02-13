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
        use_cuda: bool = True,
        # other backend specific keyword arguments
        # `chunks` and `cache` DO NOT go here, they are handled by xarray
    ) -> xr.Dataset:
        if use_cuda:  # default to nvTIFF backend
            import cupy as cp

            from cog3pio import CudaCogReader

            with cp.cuda.Stream(ptds=True):
                cog = CudaCogReader(path=filename_or_obj)
                array_: cp.ndarray = cp.from_dlpack(cog)  # 1-D Array
                x_coords, y_coords = cog.xy_coords()  # TODO consider using rasterix
                height, width = (len(y_coords), len(x_coords))
                channels: int = len(array_) // (height * width)
                # TODO make API to get proper 3-D shape directly, or use cuTENSOR
                array_ = array_.reshape(height, width, channels)  # HWC
                array = array_.transpose(2, 0, 1)  # CHW
        else:  # fallback to regular TIFF reader
            cog = CogReader(path=filename_or_obj)
            array: np.ndarray = np.from_dlpack(cog)
            x_coords, y_coords = cog.xy_coords()  # TODO consider using rasterix
            channels, _height, _width = array.shape

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
