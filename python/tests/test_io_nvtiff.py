"""
Test GeoTIFF I/O via nvTIFF backend.
"""

import pytest
from cog3pio import CudaCogReader

cp = pytest.importorskip("cupy")


def test_cudacogreader_to_dlpack():
    """
    Ensure that the CudaCogReader class's `__dlpack__` method produces a dl_tensor that
    can be read into a cupy.ndarray.
    """
    cog = CudaCogReader(
        path="https://github.com/rasterio/rasterio/raw/1.4.3/tests/data/float32.tif"
    )

    assert hasattr(cog, "__dlpack__")
    assert hasattr(cog, "__dlpack_device__")
    array = cp.from_dlpack(cog)

    assert array.shape == (6,)  # (1, 2, 3)  # band, height, width
    cp.testing.assert_allclose(
        actual=array,
        desired=cp.array(
            [[[1.41, 1.23, 0.78], [0.32, -0.23, -1.88]]], dtype=cp.float32
        ),
    )
