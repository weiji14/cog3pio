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
            path="https://github.com/rasterio/rasterio/raw/1.5.0/tests/data/float32.tif"
        )

        assert hasattr(cog, "__dlpack__")
        assert hasattr(cog, "__dlpack_device__")

        array: cp.ndarray = cp.from_dlpack(cog)

    assert array.shape == (6,)  # (1, 2, 3)  # band, height, width
    cp.testing.assert_allclose(
        actual=array,
        desired=cp.array([1.41, 1.23, 0.78, 0.32, -0.23, -1.88], dtype=cp.float32),
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
                    cog = CudaCogReader(path=tiff)
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
