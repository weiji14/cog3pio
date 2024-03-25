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

    let arr: Array3<f32> = read_geotiff(stream).unwrap();
    assert_eq!(arr.dim(), (1, 549, 549));
    assert_eq!(arr[[0, 500, 500]], 0.13482364);
}
```

### Python

```python
import numpy as np
from cog3pio import read_geotiff

array: np.ndarray = read_geotiff(
    path="https://github.com/cogeotiff/rio-tiler/raw/6.4.0/tests/fixtures/cog_nodata_nan.tif"
)
assert array.shape == (1, 549, 549)  # bands, height, width
assert array.dtype == "float32"
```

> [!NOTE]
> Currently, this crate/library only supports reading single or multi-band float32
> GeoTIFF files, i.e. other dtypes (e.g. uint16) don't work yet. See roadmap below on
> future plans.


## Roadmap

Short term (Q1 2024):
- [ ] Implement single-band GeoTIFF reader (for uint/int/float dtypes) to
      [`ndarray`](https://github.com/rust-ndarray/ndarray)
- [x] Multi-band reader (relying on
      [`image-tiff`](https://github.com/image-rs/image-tiff))
- [x] Read from remote storage (using
      [`object-store`](https://github.com/apache/arrow-rs/tree/object_store_0.9.0/object_store))

Medium term (Q2 2024):
- [x] Integration with `xarray` as a
      [`BackendEntrypoint`](https://docs.xarray.dev/en/v2024.02.0/internals/how-to-add-new-backend.html)
- [ ] Parallel reader (TBD on multi-threaded or asynchronous)
- [ ] Direct-to-GPU loading


## Related crates

- https://github.com/georust/geotiff
- https://github.com/jblindsay/whitebox-tools
- https://github.com/pka/georaster
