// Benchmark tests on reading a Cloud-optimized GeoTIFF (CoG) into memory (CPU or GPU)
//
// Libraries compared:
// - nvTIFF (Enable NVIDIA network repository and do `sudo apt install nvtiff nvcomp-cuda-13`)
// - GDAL
// - image-tiff
//
// Steps:
// - Download Sentinel-2 True-Colour Image (TCI) file (318.0MB, DEFLATE compression) from
//   https://sentinel-cogs.s3.us-west-2.amazonaws.com/sentinel-s2-l2a-cogs/37/M/BV/2024/10/S2A_37MBV_20241029_0_L2A/TCI.tif
//   to `benches/` folder.
// - Run `cargo bench` (CPU-only) or `cargo bench --features cuda` (with CUDA-enabled GPU)
//
// References:
// - https://github.com/microsoft/pytorch-cloud-geotiff-optimization/blob/5fb6d1294163beff822441829dcd63a3791b7808/configs/search.yaml#L6

use std::fs::File;
#[cfg(feature = "cuda")]
use std::sync::Arc;
#[cfg(feature = "cuda")]
use std::time::Duration;

#[cfg(feature = "cuda")]
use bytes::Bytes;
use cog3pio::io::geotiff::CogReader;
#[cfg(feature = "cuda")]
use cog3pio::io::nvtiff::CudaCogReader;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
#[cfg(feature = "cuda")]
use cudarc::driver::{CudaContext, CudaStream};
use dlpark::SafeManagedTensorVersioned;
use dlpark::traits::TensorView;
use gdal::raster::Buffer;
use gdal::{Dataset, DatasetOptions, GdalOpenFlags};
use ndarray::Array2;

// nvtiff
#[cfg(feature = "cuda")]
fn read_geotiff_nvtiff(fpath: &str) {
    let v = std::fs::read(fpath).unwrap();
    let bytes = Bytes::copy_from_slice(&v);

    let ctx: Arc<CudaContext> = CudaContext::new(0).unwrap(); // Set on GPU:0
    let cuda_stream: Arc<CudaStream> = ctx.per_thread_stream();

    let cog = CudaCogReader::new(&bytes).unwrap();
    let tensor: SafeManagedTensorVersioned = cog.dlpack(&cuda_stream).unwrap();

    assert_eq!(tensor.num_elements(), 3 * 10980 * 10980);
    // drop(cog);
    // cuda_stream.synchronize().unwrap();
}

// gdal
fn read_geotiff_gdal(fpath: &str) {
    let options = DatasetOptions {
        open_flags: GdalOpenFlags::default(),
        allowed_drivers: Some(&["LIBERTIFF"]),
        open_options: Some(&["NUM_THREADS=4"]),
        sibling_files: None,
    };
    let ds = Dataset::open_ex(fpath, options).unwrap();

    for b in 1..3 {
        let band = ds.rasterband(b).unwrap();
        let buffer: Buffer<u8> = band.read_band_as::<u8>().unwrap();
        let array: Array2<_> = buffer.to_array().unwrap();

        assert_eq!(array.shape(), [10980, 10980]);

        #[cfg(feature = "cuda")]
        {
            // Copy from CPU (host) memory to CUDA (device) memory
            let ctx: Arc<CudaContext> = CudaContext::new(0).unwrap(); // Set on GPU:0
            let cuda_stream: Arc<CudaStream> = ctx.per_thread_stream();
            let mut cuda_mem = cuda_stream.alloc_zeros::<u8>(3 * 10980 * 10980).unwrap();

            cuda_stream
                .memcpy_htod(array.as_slice().unwrap(), &mut cuda_mem)
                .unwrap();
        }
    }
}

// image-tiff
fn read_geotiff_image_tiff(fpath: &str) {
    let file = File::open(fpath).unwrap();

    let mut cog = CogReader::new(file).unwrap();
    let tensor: SafeManagedTensorVersioned = cog.dlpack().unwrap();

    assert_eq!(tensor.num_elements(), 3 * 10980 * 10980);

    #[cfg(feature = "cuda")]
    {
        // Copy from CPU (host) memory to CUDA (device) memory
        let ctx: Arc<CudaContext> = CudaContext::new(0).unwrap(); // Set on GPU:0
        let cuda_stream: Arc<CudaStream> = ctx.per_thread_stream();
        let mut cuda_mem = cuda_stream.alloc_zeros::<u8>(3 * 10980 * 10980).unwrap();

        let slice: &[u8] = tensor.as_slice_untyped();
        cuda_stream.memcpy_htod(slice, &mut cuda_mem).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_cog");

    let fsize: u64 = std::fs::metadata("benches/TCI.tif").unwrap().len();
    group.throughput(Throughput::BytesDecimal(fsize)); // 318MB filesize

    // GPU decoding using nvTIFF, reduce sample size because of CUDA memory leak
    #[cfg(feature = "cuda")]
    group
        .sample_size(10)
        .nresamples(2)
        .warm_up_time(Duration::from_millis(1))
        .measurement_time(Duration::from_secs(2));
    // .noise_threshold(0.15);
    #[cfg(feature = "cuda")]
    group.bench_with_input(
        BenchmarkId::new("0_nvTIFF_GPU", "Sentinel-2 TCI"),
        "benches/TCI.tif",
        |b, p| b.iter(|| read_geotiff_nvtiff(p)),
    );

    // CPU decoding using GDAL
    group.sample_size(30);
    group.bench_with_input(
        BenchmarkId::new("1_gdal_CPU", "Sentinel-2 TCI"),
        "benches/TCI.tif",
        |b, p| b.iter(|| read_geotiff_gdal(p)),
    );

    // CPU decoding based on image-tiff
    group.sample_size(30);
    group.bench_with_input(
        BenchmarkId::new("2_image-tiff_CPU", "Sentinel-2 TCI"),
        "benches/TCI.tif",
        |b, p| b.iter(|| read_geotiff_image_tiff(p)),
    );

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
