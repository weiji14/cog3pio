"""
Test I/O on GeoTIFF files.
"""
import tempfile
import urllib.request
import os
import pytest

from cog3pio import read_geotiff

# %%
@pytest.fixture(scope="module", name="geotiff_path")
def fixture_geotiff_path():
    """
    Filepath to a sample single-band GeoTIFF file.
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        geotiff_path = os.path.join(tmpdir, "float32.tif")
        urllib.request.urlretrieve(
            url="https://github.com/pka/georaster/raw/v0.1.0/data/tiff/float32.tif",
            filename=geotiff_path,
        )
        yield geotiff_path


@pytest.mark.benchmark
def test_read_geotiff_local(geotiff_path):
    """
    Read a GeoTIFF file from a local file path.
    """
    array = read_geotiff(path=geotiff_path)
    assert array.shape == (20, 20)
    assert array.dtype == "float32"


@pytest.mark.benchmark
def test_read_geotiff_remote():
    """
    Read a GeoTIFF file from a remote URL.
    """
    array = read_geotiff(
        path="https://github.com/pka/georaster/raw/v0.1.0/data/tiff/float32.tif"
    )
    assert array.shape == (20, 20)
    assert array.dtype == "float32"
