use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    // `stub_info` is a function defined by `define_stub_info_gatherer!` macro.
    #[cfg(feature = "pyo3")]
    {
        let stub = cog3pio::adapters::stub_info()?;
        stub.generate()?;
    }
    #[cfg(all(feature = "cuda", feature = "pyo3"))]
    {
        let stub = cog3pio::cudacog::stub_info()?;
        stub.generate()?;
    }
    Ok(())
}
