# Changelog

All notable changes to this project will be documented in this file.

---

## Unreleased

### <!-- 0 --> 🌈 Highlights

- 💥 Read into CUDA memory DLPack tensor ([#57](https://github.com/weiji14/cog3pio/pull/57))
- ⬆️ SPEC 0: Bump min version to Python 3.13 ([#66](https://github.com/weiji14/cog3pio/pull/66))

### <!-- 1 --> ✨ Features

- ✨ Implement xy_coords method on PyCudaCogReader struct ([#82](https://github.com/weiji14/cog3pio/pull/82))
- ✨ Handle dl_device and copy params in CudaCogReader `__dlpack__` method ([#77](https://github.com/weiji14/cog3pio/pull/77))
- ✨ Create Transform trait to get AffineTransform and xy_coords ([#70](https://github.com/weiji14/cog3pio/pull/70))
- ✨ Python bindings for CudaCogReader ([#58](https://github.com/weiji14/cog3pio/pull/58))
- ✨ Decode GeoTIFF images to CUDA stream ([#27](https://github.com/weiji14/cog3pio/pull/27))

### <!-- 2 --> 🐛 Bug Fixes

- 🐛 Fix Cargo.toml metadata description to enable crates.io publishing ([#51](https://github.com/weiji14/cog3pio/pull/51))

### <!-- 3 --> 🏭 Refactors

- 🔥 Remove cog3pio backend engine, drop xarray dependency ([#89](https://github.com/weiji14/cog3pio/pull/89))
- 🚚 Move benchmark,docs,tests extras into dependency-groups ([#86](https://github.com/weiji14/cog3pio/pull/86))
- 🚚 Switch docs build to use Zensical ([#73](https://github.com/weiji14/cog3pio/pull/73))
- ♻️ Refactor to pass in per-thread default stream to dlpack method ([#67](https://github.com/weiji14/cog3pio/pull/67))

### <!-- 4 --> 📝 Documentation

- 📝 More docs for nvTIFF/CUDA backend install and usage ([#90](https://github.com/weiji14/cog3pio/pull/90))
- 🔍️ Add description, readme and urls to pyproject.toml metadata ([#87](https://github.com/weiji14/cog3pio/pull/87))
- 🏷️ Generate pyi stub files for type hints ([#72](https://github.com/weiji14/cog3pio/pull/72))
- 🔍️ Add shield badges to main README.md ([#52](https://github.com/weiji14/cog3pio/pull/52))

### <!-- 5 --> 🧰 Maintenance

- 👷 Build wheels for riscv64 ([#88](https://github.com/weiji14/cog3pio/pull/88))
- 🚨 Tidy up Cargo.toml and pyproject.toml files ([#84](https://github.com/weiji14/cog3pio/pull/84))
- 🔒️ Configure trusted publishing to crates.io ([#83](https://github.com/weiji14/cog3pio/pull/83))
- 👷 Install zizmor directly through cargo ([#81](https://github.com/weiji14/cog3pio/pull/81))
- 👷 Build wheels with cuda feat flag properly enabled ([#80](https://github.com/weiji14/cog3pio/pull/80))
- 🚩 Set pyo3 and not(doctest) flags appropriately ([#78](https://github.com/weiji14/cog3pio/pull/78))
- 👷 Use cheaper CI runners and set concurrency limits ([#76](https://github.com/weiji14/cog3pio/pull/76))
- 🚨 Check building for docs.rs ([#75](https://github.com/weiji14/cog3pio/pull/75))
- ⬆️ Bump nvtiff-sys from 0.2.0 to 0.2.1, tiff from 0.10.0 to 0.11.2 ([#68](https://github.com/weiji14/cog3pio/pull/68))
- ⬆️ Bump gdal from 8b9e049 to 0.19.0 ([#65](https://github.com/weiji14/cog3pio/pull/65))
- ⬆️ Bump nvtiff-sys from 0.1.2 to 0.2.0, cudarc from 0.17.7 to 0.18.2 ([#64](https://github.com/weiji14/cog3pio/pull/64))
- ⬆️ Bump ndarray from 0.16.1 to 0.17.1 ([#63](https://github.com/weiji14/cog3pio/pull/63))
- ➖ Turn tokio into optional dependency under pyo3 flag ([#62](https://github.com/weiji14/cog3pio/pull/62))
- 👷 Track Rust performance benchmarks on CI ([#61](https://github.com/weiji14/cog3pio/pull/61))
- ⬆️ Bump pyo3 from 0.25.0 to 0.27.1, numpy from 0.25.0 to 0.27.0 ([#54](https://github.com/weiji14/cog3pio/pull/54), [#60](https://github.com/weiji14/cog3pio/pull/60))
- 🚩 Gate python bindings behind pyo3 feature flag ([#59](https://github.com/weiji14/cog3pio/pull/59))
- 👷 Run CI on macos-15-intel and macos-14 ([#56](https://github.com/weiji14/cog3pio/pull/56))
- ⬆️ Bump object_store from 0.9.0 to 0.12.3, url from 2.5.0 to 2.5.7 ([#55](https://github.com/weiji14/cog3pio/pull/55))

### 🧑‍🤝‍🧑 Contributors

- [@weiji14](https://github.com/weiji14)

---

## [0.0.1] - 2025-06-28

### <!-- 0 --> 🌈 Highlights

- 💥 Read into DLPack tensor ([#32](https://github.com/weiji14/cog3pio/pull/32))
- ✨ Implement cog3pio xarray BackendEntrypoint ([#14](https://github.com/weiji14/cog3pio/pull/14))

### <!-- 1 --> ✨ Features

- ✨ Support decoding ZSTD compressed and half-precision TIFFs ([#46](https://github.com/weiji14/cog3pio/pull/46))
- ✨ Support reading 3-band RGB images ([#31](https://github.com/weiji14/cog3pio/pull/31))
- ✨ Support reading uint/int/float dtypes ([#18](https://github.com/weiji14/cog3pio/pull/18))
- ✨ Support reading multi-band GeoTIFF files ([#13](https://github.com/weiji14/cog3pio/pull/13))
- ✨ Implement PyCogReader struct with new and to_numpy methods ([#12](https://github.com/weiji14/cog3pio/pull/12))
- ✨ CogReader ndarray method to decode TIFF into an ndarray Array ([#10](https://github.com/weiji14/cog3pio/pull/10))
- ✨ Get affine transformation from GeoTIFF ([#8](https://github.com/weiji14/cog3pio/pull/8))
- ✨ Read GeoTIFF files from remote urls via object_store ([#5](https://github.com/weiji14/cog3pio/pull/5))
- ✨ A read_geotiff function for reading GeoTIFF into ndarray ([#3](https://github.com/weiji14/cog3pio/pull/3))

### <!-- 3 --> 🏭 Refactors

- ♻️ Refactor to return 3D arrays of shape (band, height, width) ([#11](https://github.com/weiji14/cog3pio/pull/11))
- 🚚 Move pyo3 functions under src/python/adapters.rs ([#9](https://github.com/weiji14/cog3pio/pull/9))
- 🎨 Initial CogReader struct with decoder field ([#7](https://github.com/weiji14/cog3pio/pull/7))
- ♻️ Refactor unit test to be non-square ([#6](https://github.com/weiji14/cog3pio/pull/6))

### <!-- 4 --> 📝 Documentation

- 📝 Move installation and example commands into separate pages ([#47](https://github.com/weiji14/cog3pio/pull/47))
- 📝 Initialize Python documentation page ([#35](https://github.com/weiji14/cog3pio/pull/35))

### <!-- 5 --> 🧰 Maintenance

- 👷 GitHub Actions CI workflow to publish to crates.io ([#49](https://github.com/weiji14/cog3pio/pull/49))
- 🔧 Switch changelog generator config from git-cliff to release-plz ([#48](https://github.com/weiji14/cog3pio/pull/48))
- 📌 Unpin sphinx-ext-mystmd in docs extras ([#44](https://github.com/weiji14/cog3pio/pull/44))
- 🔊 Enable verbose logging for pypa/gh-action-pypi-publish ([#42](https://github.com/weiji14/cog3pio/pull/42))
- 👷 Upload to TestPyPI on prerelease and release tags ([#40](https://github.com/weiji14/cog3pio/pull/40))
- 👷 Adjust CI workflow conditions for release trigger ([#38](https://github.com/weiji14/cog3pio/pull/38))
- 🔧 Configure readthedocs documentation build ([#36](https://github.com/weiji14/cog3pio/pull/36))
- 👷 Build free-threaded wheels on CI and upload to TestPyPI ([#34](https://github.com/weiji14/cog3pio/pull/34))
- 🚨 Setup CI to lint using cargo fmt + clippy pedantic fixes ([#33](https://github.com/weiji14/cog3pio/pull/33))
- 👷 Run aarch64 CI tests on ubuntu-24.04-arm ([#30](https://github.com/weiji14/cog3pio/pull/30))
- ⬆️ SPEC 0: Bump min version to Python 3.12, NumPy 2.0, xarray 2023.12.0 ([#29](https://github.com/weiji14/cog3pio/pull/29))
- 📌 Pin MSRV to 1.78.0 ([#28](https://github.com/weiji14/cog3pio/pull/28))
- ⬆️ Bump pyo3 from 0.20.3 to 0.25.0, numpy from 0.20.0 to 0.25.0 ([#15](https://github.com/weiji14/cog3pio/pull/15), [#19](https://github.com/weiji14/cog3pio/pull/19), [#21](https://github.com/weiji14/cog3pio/pull/21), [#25](https://github.com/weiji14/cog3pio/pull/25))
- 🔒️ Add zizmor to statically analyze GitHub Actions workflows ([#24](https://github.com/weiji14/cog3pio/pull/24))
- 👷 Run CI on ubuntu-24.04, macos-15, windows-2025 ([#23](https://github.com/weiji14/cog3pio/pull/23))
- 🚨 Setup CI to run linting using cargo clippy ([#22](https://github.com/weiji14/cog3pio/pull/22))
- ⬆️ Bump geo from 0.28.0 rev 481196b to 0.29.0 ([#20](https://github.com/weiji14/cog3pio/pull/20))
- 👷 Setup CI job matrix to run cargo test ([#17](https://github.com/weiji14/cog3pio/pull/17))
- 👷 Setup benchmark workflow with pytest-codspeed ([#4](https://github.com/weiji14/cog3pio/pull/4))
- 👷 Setup GitHub Actions Continuous Integration tests ([#2](https://github.com/weiji14/cog3pio/pull/2))
- 🌱 Initialize Cargo.toml and pyproject.toml with maturin ([#1](https://github.com/weiji14/cog3pio/pull/1))

### 🧑‍🤝‍🧑 Contributors

- [@weiji14](https://github.com/weiji14)
