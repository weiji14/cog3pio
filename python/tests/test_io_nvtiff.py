"""
Test GeoTIFF I/O via nvTIFF backend.
"""

import pytest

cp = pytest.importorskip("cupy")

from cog3pio import CudaCogReader


def test_cudacogreader_to_dlpack():
    """
    Ensure that the CudaCogReader class's `__dlpack__` method produces a dl_tensor that
    can be read into a cupy.ndarray.
    """

    with cp.cuda.Stream(null=True):
        _cog = CudaCogReader(
            path="https://github.com/rasterio/rasterio/raw/1.4.3/tests/data/float32.tif"
        )
        print(cp.cuda.get_current_stream())
    print(cp.cuda.get_current_stream())
    cog = CudaCogReader(
        path="https://github.com/rasterio/rasterio/raw/1.4.3/tests/data/float32.tif"
    )
    print(cp.cuda.get_current_stream())
    assert hasattr(cog, "__dlpack__")
    assert hasattr(cog, "__dlpack_device__")
    array = cp.from_dlpack(cog)

    assert array.shape == (6,)  # (1, 2, 3)  # band, height, width
    cp.testing.assert_allclose(
        actual=array,
        desired=cp.array([[1.41, 1.23, 0.78, 0.32, -0.23, -1.88]], dtype=cp.float32),
    )
