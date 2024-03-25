"""
Test I/O on GeoTIFF files.
"""
import os
import tempfile
import urllib.request

import numpy as np
import pytest

from cog3pio import CogReader, read_geotiff


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
    Read a single-band GeoTIFF file from a local file path.
    """
    array = read_geotiff(path=geotiff_path)
    assert array.shape == (1, 20, 20)
    assert array.dtype == "float32"


@pytest.mark.benchmark
def test_read_geotiff_remote():
    """
    Read a single-band GeoTIFF file from a remote URL.
    """
    array = read_geotiff(
        path="https://github.com/pka/georaster/raw/v0.1.0/data/tiff/float32.tif"
    )
    assert array.shape == (1, 20, 20)
    assert array.dtype == "float32"


@pytest.mark.benchmark
def test_read_geotiff_multi_band():
    """
    Read a multi-band GeoTIFF file from a remote URL.
    """
    array = read_geotiff(
        path="https://github.com/locationtech/geotrellis/raw/v3.7.1/raster/data/one-month-tiles-multiband/result.tif"
    )
    assert array.shape == (2, 512, 512)
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


def test_read_geotiff_unsupported_colortype():
    """
    Check that a ValueError is raised when an unsupported GeoTIFF (with ColorType::RGB)
    is passed to read_geotiff.
    """
    with pytest.raises(
        ValueError,
        match="The Decoder does not support the image format "
        r"`RGBPalette with \[8\] bits per sample is unsupported",
    ):
        read_geotiff(
            path="https://github.com/GenericMappingTools/gmtserver-admin/raw/caf0dbd015f0154687076dd31dc8baff62c95040/cache/earth_day_HD.tif"
        )


def test_read_geotiff_unsupported_dtype():
    """
    Check that a ValueError is raised when an unsupported GeoTIFF (of ComplexInt16 type)
    is passed to read_geotiff.
    """
    with pytest.raises(
        ValueError,
        match="The Decoder does not support the image format "
        r"`Sample format \[Unknown\(5\)\] is unsupported",
    ):
        read_geotiff(
            path="https://github.com/corteva/rioxarray/raw/0.15.1/test/test_data/input/cint16.tif"
        )


def test_CogReader_to_numpy():
    """
    Ensure that the CogReader class's `to_numpy` method produces a numpy.ndarray output.
    """
    reader = CogReader(
        path="https://github.com/rasterio/rasterio/raw/1.3.9/tests/data/float32.tif"
    )
    array = reader.to_numpy()
    assert array.shape == (1, 2, 3)  # band, height, width
    np.testing.assert_equal(
        actual=array,
        desired=np.array(
            [[[1.41, 1.23, 0.78], [0.32, -0.23, -1.88]]], dtype=np.float32
        ),
    )
