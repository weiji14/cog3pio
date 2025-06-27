# cog3pio

Cloud-optimized GeoTIFF ... Parallel I/O

Yet another attempt at creating a GeoTIFF reader, in Rust, with Python bindings.


## Usage

### Rust

#### Ndarray

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
    let arr: Array3<f32> = read_geotiff::<f32, _>(stream).unwrap();
    assert_eq!(arr.dim(), (1, 549, 549));
    assert_eq!(arr[[0, 500, 500]], 0.13482364);
}
```

#### DLPack

```rust
use std::io::Cursor;

use bytes::Bytes;
use cog3pio::io::geotiff::CogReader;
use dlpark::ffi::DataType;
use dlpark::prelude::TensorView;
use dlpark::SafeManagedTensorVersioned;
use object_store::path::Path;
use object_store::{parse_url, GetResult, ObjectStore};
use tokio;
use url::Url;

#[tokio::main]
async fn main() {
    let cog_url: &str =
        "https://github.com/cogeotiff/rio-tiler/raw/7.8.0/tests/fixtures/cog_dateline.tif";
    let tif_url: Url = Url::parse(cog_url).unwrap();
    let (store, location): (Box<dyn ObjectStore>, Path) = parse_url(&tif_url).unwrap();

    let stream: Cursor<Bytes> = {
        let result: GetResult = store.get(&location).await.unwrap();
        let bytes: Bytes = result.bytes().await.unwrap();
        Cursor::new(bytes)
    };

    // Read GeoTIFF into a dlpark::versioned::SafeManagedTensorVersioned
    let mut cog = CogReader::new(stream).unwrap();
    let tensor: SafeManagedTensorVersioned = cog.dlpack().unwrap();
    assert_eq!(tensor.shape(), [1, 2355, 2325]);
    assert_eq!(tensor.data_type(), &DataType::U16);
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
    filename_or_obj="https://github.com/cogeotiff/rio-tiler/raw/7.8.0/tests/fixtures/cog_dateline.tif",
    engine="cog3pio",
)
assert dataarray.sizes == {'band': 1, 'y': 2355, 'x': 2325}
assert dataarray.dtype == "uint16"
```

> [!NOTE]
> The cog3pio python library's `read_geotiff` function supports reading single or
> multi-band GeoTIFF files into a float32 array only. If you wish to read into other
> dtypes (e.g. uint16), please use `xarray.open_dataarray` with `engine="cog3pio"`, or
> use the cog3pio Rust crate's `read_geotiff` function which supports reading into
> different dtypes via a turbofish operator!


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
      on [`geotiff`](https://github.com/georust/geotiff) crate)

Longer term (2025):
- [ ] Parallel reader (TBD on multi-threaded or asynchronous)
- [ ] Direct-to-GPU loading


## Related crates

- https://github.com/developmentseed/async-tiff
- https://github.com/feefladder/tiff2
- https://github.com/georust/geotiff
- https://github.com/jblindsay/whitebox-tools
- https://github.com/pka/georaster
