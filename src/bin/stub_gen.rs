use pyo3_stub_gen::Result;

#[cfg(feature = "pyo3")]
fn main() -> Result<()> {
    // `stub_info` is a function defined by `define_stub_info_gatherer!` macro.
    let stub = cog3pio::stub_info()?;
    stub.generate()?;
    Ok(())
}
