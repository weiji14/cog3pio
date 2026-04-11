# ⚙️ *cog3pio* - Cloud-optimized GeoTIFF ... Parallel I/O

*Practice the Three Acts of CoGnizance - Use GeoTIFF-tiles, Be GDAL-less, Go GPU-accelerated*

## Installation

### Rust 🦀

Stay minimal and pure, or treat yourself to more
[features](https://docs.rs/crate/cog3pio/latest/features).

```bash
cargo add cog3pio
```

### Python 🐍

The minimal CPU-only wheels using [image-tiff](https://crates.io/crates/tiff) backend.

```bash
pip install cog3pio
```

add an extra CUDA-based [nvTIFF](https://crates.io/crates/nvtiff-sys) backend
(pre-built wheels only on Linux `x86_64` and `aarch64`). Requires a
[patched nvTIFF](#extra-instructions) library.

```
pip install cog3pio[cuda]
```

Alternatively, fetch it from
[conda-forge](https://anaconda.org/channels/conda-forge/packages/cog3pio/overview),
which will include the CUDA features if you are on a supported platform.

```bash
conda install --channel conda-forge cog3pio
```

!!! tip
    The API for this crate/library is still unstable and subject to change, so you may
    want to pin to a specific git commit using either:
    
    - `cargo add --git https://github.com/weiji14/cog3pio.git --rev <sha>`
    - `pip install git+https://github.com/weiji14/cog3pio.git@<sha>`
    
    where `<sha>` is a commit hashsum obtained from
    <https://github.com/weiji14/cog3pio/commits/main>

### Extra instructions

For Linux users who have a CUDA GPU, go and
[download and install nvTIFF](https://developer.nvidia.com/nvtiff-downloads?target_os=Linux)
using your system's package manager (recommended).

```bash
# ... set up nvidia sources before running below
apt -y install nvtiff-cuda-13  # debian/ubuntu
dnf install -y nvtiff-cuda-13  # rocky/rhel
```

or via [conda-forge](https://anaconda.org/channels/conda-forge/packages/libnvtiff-dev)
(not so recommended unless you know what you're doing)

```bash
conda install --channel conda-forge libnvtiff-dev
```

then locate the `nvtiff.h` header file and apply this patch.

```bash
sed --in-place "s/nvtiffTagDataType type/enum nvtiffTagDataType type/g" /usr/include/nvtiff.h
```

Getting frustrated?
Open an [issue](https://github.com/weiji14/cog3pio/issues) (or find one already there)!
