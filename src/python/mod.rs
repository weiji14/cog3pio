/// Adapter interface from Rust to Python
#[cfg(not(doctest))]
pub mod adapters;
/// `CudaCogReader` adapter
#[cfg(all(feature = "cuda", not(doctest)))]
mod cudacog;
