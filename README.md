# cog3pio

Cloud-optimized GeoTIFF ... Parallel I/O

Yet another attempt at creating a GeoTIFF reader, in Rust, with Python bindings.


## Installation

### Rust

    cargo add --git https://github.com/weiji14/cog3pio.git

### Python

    pip install git+https://github.com/weiji14/cog3pio.git

> [!TIP]
> The API for this crate/library is still unstable and subject to change, so you may
> want to pin to a specific git commit using either:
> - `cargo add --git https://github.com/weiji14/cog3pio.git --rev <sha>`
> - `pip install git+https://github.com/weiji14/cog3pio.git@<sha>`
>
> where `<sha>` is a commit hashsum obtained from
> https://github.com/weiji14/cog3pio/commits/main


## Usage

### Rust

```rust
use std::io::Cursor;

use bytes::Bytes;
use cog3pio::io::geotiff::read_geotiff;
use ndarray::Array3;
use object_store::path::Path;
use object_store::{parse_url, GetResult, ObjectStore};
use tokio;
use url::Url;

#[tokio::main]
async fn main() {
    let cog_url: &str =
        "https://github.com/cogeotiff/rio-tiler/raw/6.4.0/tests/fixtures/cog_nodata_nan.tif";
    let tif_url: Url = Url::parse(cog_url).unwrap();
    let (store, location): (Box<dyn ObjectStore>, Path) = parse_url(&tif_url).unwrap();

    let stream: Cursor<Bytes> = {
        let result: GetResult = store.get(&location).await.unwrap();
        let bytes: Bytes = result.bytes().await.unwrap();
        Cursor::new(bytes)
    };

    // Read GeoTIFF into an ndarray::Array
    let arr: Array3<f32> = read_geotiff(stream).unwrap();
    assert_eq!(arr.dim(), (1, 549, 549));
    assert_eq!(arr[[0, 500, 500]], 0.13482364);
}
```

### Python

#### NumPy

```python
import numpy as np
from cog3pio import read_geotiff

# Read GeoTIFF into a numpy array
array: np.ndarray = read_geotiff(
    path="https://github.com/cogeotiff/rio-tiler/raw/6.4.0/tests/fixtures/cog_nodata_nan.tif"
)
assert array.shape == (1, 549, 549)  # bands, height, width
assert array.dtype == "float32"
```

#### Xarray

```python
import xarray as xr

# Read GeoTIFF into an xarray.DataArray
dataarray: xr.DataArray = xr.open_dataarray(
    filename_or_obj="https://github.com/cogeotiff/rio-tiler/raw/6.4.1/tests/fixtures/cog_nodata_nan.tif",
    engine="cog3pio",
)
assert dataarray.sizes == {'band': 1, 'y': 549, 'x': 549}
assert dataarray.dtype == "float32"
```

> [!NOTE]
> Currently, the Python library supports reading single or multi-band GeoTIFF files into
> a float32 array only, i.e. other dtypes (e.g. uint16) don't work yet. There is support
> for reading into different dtypes in the Rust crate via a turbofish operator though!


## Roadmap

Short term (Q1 2024):
- [x] Multi-band reader to [`ndarray`](https://github.com/rust-ndarray/ndarray) (relying
      on [`image-tiff`](https://github.com/image-rs/image-tiff))
- [x] Read from HTTP remote storage (using
      [`object-store`](https://github.com/apache/arrow-rs/tree/object_store_0.9.0/object_store))

Medium term (Q2-Q4 2024):
- [x] Integration with `xarray` as a
      [`BackendEntrypoint`](https://docs.xarray.dev/en/v2024.02.0/internals/how-to-add-new-backend.html)
- [x] Implement single-band GeoTIFF reader for multiple dtypes (uint/int/float) (based
      on [`geotiff`](https://github.com/georust/geotiff) crate, Rust-only)

Longer term (2025):
- [ ] Parallel reader (TBD on multi-threaded or asynchronous)
- [ ] Direct-to-GPU loading


## Related crates

- https://github.com/developmentseed/aiocogeo-rs
- https://github.com/georust/geotiff
- https://github.com/jblindsay/whitebox-tools
- https://github.com/pka/georaster
