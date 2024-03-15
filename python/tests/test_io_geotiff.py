"""
Test I/O on GeoTIFF files.
"""
import os
import tempfile
import urllib.request

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
    assert array.shape == (1, 20, 20)
    assert array.dtype == "float32"


@pytest.mark.benchmark
def test_read_geotiff_remote():
    """
    Read a GeoTIFF file from a remote URL.
    """
    array = read_geotiff(
        path="https://github.com/pka/georaster/raw/v0.1.0/data/tiff/float32.tif"
    )
    assert array.shape == (1, 20, 20)
    assert array.dtype == "float32"


def test_read_geotiff_invalid_filepath():
    """
    Check that a ValueError is raised when an invalid filepath is passed to read_geotiff.
    """
    with pytest.raises(ValueError, match=r"Cannot parse path: \\invalid\\path"):
        read_geotiff(path=r"\invalid\path")


def test_read_geotiff_invalid_remote_url():
    """
    Check that a ValueError is raised when an invalid remote url is passed to read_geotiff.
    """
    with pytest.raises(ValueError, match="Cannot parse url: protocol://file.ext"):
        read_geotiff(path="protocol://file.ext")


def test_read_geotiff_missing_url():
    """
    Check that a FileNotFoundError is raised when a url pointing to a non-existent file
    is passed to read_geotiff.
    """
    with pytest.raises(
        FileNotFoundError, match="Cannot find file: https://example.com/geo.tif"
    ):
        read_geotiff(path="https://example.com/geo.tif")


def test_read_geotiff_unsupported_dtype():
    """
    Check that a ValueError is raised when an unsupported GeoTIFF (of ComplexInt16 type)
    is passed to read_geotiff.
    """
    with pytest.raises(
        ValueError,
        match="The Decoder does not support the image format ",
    ):
        read_geotiff(
            path="https://github.com/corteva/rioxarray/raw/0.15.1/test/test_data/input/cint16.tif"
        )
