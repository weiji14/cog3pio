"""
Test GeoTIFF I/O via nvTIFF backend.
"""

import sys

import pytest

cp = pytest.importorskip("cupy")

import cupyx.profiler
import numpy as np
from cog3pio import CudaCogReader

try:
    import rasterio

    HAS_RASTERIO = True
except ImportError:
    HAS_RASTERIO = False


# %%
def test_cudacogreader_to_dlpack():
    """
    Ensure that the CudaCogReader class's `__dlpack__` method produces a dl_tensor that
    can be read into a cupy.ndarray on the default legacy stream.
    """
    with cp.cuda.Stream(null=True):  # use legacy default stream
        cog = CudaCogReader(
            path="https://github.com/rasterio/rasterio/raw/1.5.0/tests/data/float32.tif",
            device_id=0,
        )

        assert hasattr(cog, "__dlpack__")
        assert hasattr(cog, "__dlpack_device__")

        array: cp.ndarray = cp.from_dlpack(cog)

    assert array.shape == (6,)  # (1, 2, 3)  # band, height, width
    cp.testing.assert_allclose(
        actual=array,
        desired=cp.array([1.41, 1.23, 0.78, 0.32, -0.23, -1.88], dtype=cp.float32),
    )


def test_cudacogreader_xy_coords():
    """
    Ensure that the CudaCogReader class's `xy_coords` method produces two numpy.ndarray
    objects representing the GeoTIFF's x- and y- coordinates.
    """
    cog = CudaCogReader(
        path="https://github.com/blacha/cogeotiff/raw/core-v9.4.0/packages/core/data/DEM_BS28_2016_1000_1141.tif",
        device_id=0,
    )
    x_coords, y_coords = cog.xy_coords()
    np.testing.assert_equal(
        actual=x_coords,
        desired=np.linspace(
            start=1679617.031,
            stop=1679680.031,
            num=63,
            endpoint=False,
            dtype=np.float64,
        ),
    )
    np.testing.assert_equal(
        actual=y_coords,
        desired=np.linspace(
            start=5362323.781,
            stop=5362079.781,
            num=244,
            endpoint=False,
            dtype=np.float64,
        ),
    )


@pytest.mark.benchmark
@pytest.mark.parametrize(
    "engine",
    [
        "cog3pio",
        pytest.param(
            "rasterio",
            marks=pytest.mark.skipif(
                condition=not HAS_RASTERIO, reason="Could not import 'rasterio'"
            ),
        ),
    ],
)
def test_benchmark_tocupy(engine):
    """
    Benchmark cog3pio.CudaCogReader decoding semi-directly to cupy.ndarray, against
    equivalent rasterio.open to numpy.ndarray, followed by host to device copy to
    cupy.ndarray.
    """
    tiff: str = "https://github.com/developmentseed/titiler/raw/1.1.0/src/titiler/core/tests/fixtures/TCI.tif"

    match engine:
        case "cog3pio":

            def cog_to_cupy():
                with cp.cuda.Stream(ptds=True):  # use per-thread default stream
                    cog = CudaCogReader(path=tiff, device_id=0)
                    array: cp.ndarray = cp.from_dlpack(cog)
                    assert array.shape == (3616812,)

        case "rasterio":

            def cog_to_cupy():
                with rasterio.open(fp=tiff) as dataset:
                    bands: np.ndarray = dataset.read(indexes=[1, 2, 3])
                    array: cp.ndarray = cp.asarray(a=bands)
                    assert array.shape == (3, 1098, 1098)

    # Run benchmark with GPU and CPU timings, report to stdout
    perf = cupyx.profiler.benchmark(func=cog_to_cupy, n_repeat=1)
    perf.name += f"[{engine}]\t"
    print(perf, file=sys.stdout)
