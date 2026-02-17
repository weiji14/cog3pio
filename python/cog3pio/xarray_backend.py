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

    When using :py:func:`xarray.open_dataarray` with ``engine="cog3pio"``, the optional
    ``device`` parameter can be set to a ``(device_type, device_id)`` tuple, conforming
    to [DLPack's device convention](https://data-apis.org/array-api/2024.12/API_specification/generated/array_api.array.__dlpack_device__.html).
    For example:

    - ``(2, 0)``: for reading into a CUDA GPU at device_id 0 (Default).
    - ``(1, 0)`` or ``None``: for reading into CPU.

    Examples
    --------
    Read a GeoTIFF from a HTTP url into an [xarray.DataArray][]:

    >>> import xarray as xr
    ...
    >>> # Read GeoTIFF into an xarray.DataArray
    >>> dataarray: xr.DataArray = xr.open_dataarray(
    ...     filename_or_obj="https://github.com/OSGeo/gdal/raw/v3.11.0/autotest/gcore/data/byte_zstd.tif",
    ...     engine="cog3pio",
    ...     device=(2, 0),  # CUDA device (2), GPU 0
    ... )
    >>> dataarray.sizes
    Frozen({'band': 1, 'y': 20, 'x': 20})
    >>> dataarray.dtype
    dtype('uint8')

    """

    description = "Use .tif files in Xarray"
    open_dataset_parameters = ("filename_or_obj", "drop_variables", "device")
    url = "https://github.com/weiji14/cog3pio"

    def open_dataset(
        self,
        filename_or_obj: str,  # type: ignore[override]
        *,
        drop_variables=None,
        device: tuple[int, int] | None = (2, 0),
        # other backend specific keyword arguments
        # `chunks` and `cache` DO NOT go here, they are handled by xarray
        mask_and_scale=None,
    ) -> xr.Dataset:
        """
        Backend open_dataset method used by Xarray in [xarray.open_dataset][].

        Parameters
        ----------
        filename_or_obj : str
            File path or url to a TIFF (.tif) image file that can be read by the
            nvTIFF or image-tiff backend library.
        device : (int, int) | None
            Device on which to place the created array, given as a tuple in the form of
            (device_type, device_id), corresponding to
            [DLPack's device convention](https://data-apis.org/array-api/2024.12/API_specification/generated/array_api.array.__dlpack_device__.html).
            Default is (2, 0) which means device_type='cuda', device_id=0. Pass
            `device=None` or `device=(1, 0)` to use the CPU fallback.

        Returns
        -------
        xarray.Dataset

        """
        if device is None:
            device = (1, 0)  # CPU device fallback
        # TODO handle device.__dlpack_device__ ?
        # https://github.com/data-apis/array-api/issues/972
        device_type, device_id = device
        match device_type:
            case 1:  # CPU (image-tiff backend)
                cog = CogReader(path=filename_or_obj)
                array: np.ndarray = np.from_dlpack(cog)
                x_coords, y_coords = cog.xy_coords()  # TODO consider using rasterix
                channels, _height, _width = array.shape
            case 2:  # CUDA (nvTIFF backend)
                import cupy as cp

                from cog3pio import CudaCogReader

                with cp.cuda.Stream(ptds=True):
                    cog = CudaCogReader(path=filename_or_obj, device_id=device_id)
                    array_: cp.ndarray = cp.from_dlpack(cog)  # 1-D Array
                    x_coords, y_coords = cog.xy_coords()  # TODO consider using rasterix
                    height, width = (len(y_coords), len(x_coords))
                    channels: int = len(array_) // (height * width)
                    # TODO make API to get proper 3-D shape directly, or use cuTENSOR
                    array_ = array_.reshape(height, width, channels)  # HWC
                    array = array_.transpose(2, 0, 1)  # CHW
            case _:
                msg = (
                    "Currently only support decoding to DLPack device_type 2 (CUDA) or "
                    f"1 (CPU), but got {device_type}"
                )
                raise NotImplementedError(msg)

        dataarray: xr.DataArray = xr.DataArray(
            data=array,
            coords={
                "band": np.arange(channels, dtype=np.uint8),
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
