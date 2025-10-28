"""
cog3pio - Cloud-optimized GeoTIFF ... Parallel I/O
"""

from importlib.metadata import version

from .cog3pio import CogReader, CudaCogReader, read_geotiff  # noqa: F401

__doc__ = cog3pio.__doc__
__version__ = version("cog3pio")  # e.g. 0.1.2.dev3+g0ab3cd78

if hasattr(cog3pio, "__all__"):
    __all__ = cog3pio.__all__
