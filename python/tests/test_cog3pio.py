from cog3pio import __doc__, __version__
from packaging.version import Version


def test_doc():
    assert __doc__ == "cog3pio - Cloud-optimized GeoTIFF ... Parallel I/O."


def test_version():
    assert Version(version=__version__) >= Version(version="0.0.1")
