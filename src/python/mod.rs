/// Adapter interface from Rust to Python
pub mod adapters;
/// `CudaCogReader` adapter
#[cfg(all(feature = "cuda", not(doctest)))]
pub mod cudacog;
