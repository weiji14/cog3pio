# Changelog

All notable changes to this project will be documented in this file.

---

## Unreleased

### <!-- 0 --> 🌈 Highlights

- 💥 Read into DLPack tensor ([#32](https://github.com/weiji14/cog3pio/pull/32))
- ✨ Implement cog3pio xarray BackendEntrypoint ([#14](https://github.com/weiji14/cog3pio/pull/14))

### <!-- 1 --> ✨ Features

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

- 📝 Initialize Python documentation page ([#35](https://github.com/weiji14/cog3pio/pull/35))

### <!-- 5 --> 🧰 Maintenance

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
