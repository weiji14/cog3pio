use std::ffi::c_void;
use std::sync::Arc;

use bytes::Bytes;
use cudarc::driver::{CudaSlice, CudaStream, DevicePtr};
use nvtiff_sys::{
    nvtiffDecodeCheckSupported, nvtiffDecodeImage, nvtiffDecodeParams, nvtiffDecoder,
    nvtiffDecoderCreateSimple, nvtiffFileInfo, nvtiffStatus_t, nvtiffStream, nvtiffStreamCreate,
    nvtiffStreamGetFileInfo, nvtiffStreamParse,
};

/// nvTIFF decoder
pub(crate) struct CudaCogReader {
    tiff_stream: *mut nvtiffStream,
    file_info: nvtiffFileInfo,
}

impl CudaCogReader {
    /// Create a new Cloud-optimized GeoTIFF decoder that decodes from a stream buffer
    pub fn new(byte_stream: &Bytes) -> Self {
        // Step 0: Init TIFF stream on host (CPU)
        let mut host_stream = std::mem::MaybeUninit::uninit();
        let mut tiff_stream: *mut nvtiffStream = host_stream.as_mut_ptr();

        let status_cpustream: nvtiffStatus_t::Type =
            unsafe { nvtiffStreamCreate(&mut tiff_stream) };
        dbg!(status_cpustream);
        assert_eq!(status_cpustream, nvtiffStatus_t::NVTIFF_STATUS_SUCCESS);

        // Step 1: Parse the TIFF data from byte stream buffer
        let status_parse: u32 =
            unsafe { nvtiffStreamParse(byte_stream.as_ptr(), usize::MAX, tiff_stream) };
        dbg!(status_parse);
        assert_eq!(status_parse, nvtiffStatus_t::NVTIFF_STATUS_SUCCESS);

        // Step 2: Extract file-level metadata information from the TIFF stream
        let mut file_info = nvtiffFileInfo::default();
        let status_fileinfo: u32 = unsafe { nvtiffStreamGetFileInfo(tiff_stream, &mut file_info) };
        dbg!(status_fileinfo);
        dbg!(file_info);
        assert_eq!(status_fileinfo, nvtiffStatus_t::NVTIFF_STATUS_SUCCESS);

        Self {
            tiff_stream,
            file_info,
        }
    }

    /// Decode GeoTIFF image to a `CudaSlice` (`Vec<u8>` on a CUDA device)
    pub fn to_cuda(&self, stream: &Arc<CudaStream>) -> CudaSlice<u8> {
        // Step 1b: Init CUDA stream on device (GPU)
        let cuda_stream: *mut nvtiff_sys::CUstream_st = stream.cu_stream().cast::<_>();

        // Step 1c: Init decoder handle
        let mut decoder_handle = std::mem::MaybeUninit::zeroed();
        let mut nvtiff_decoder: *mut nvtiffDecoder = decoder_handle.as_mut_ptr();

        let status_decoder: u32 =
            unsafe { nvtiffDecoderCreateSimple(&mut nvtiff_decoder, cuda_stream) };
        dbg!(status_decoder);
        assert_eq!(status_decoder, nvtiffStatus_t::NVTIFF_STATUS_SUCCESS);

        // Step 3a: Check if image is supported first
        let mut params = std::mem::MaybeUninit::zeroed();
        let decode_params: *mut nvtiffDecodeParams = params.as_mut_ptr();
        let status_check: u32 = unsafe {
            nvtiffDecodeCheckSupported(
                self.tiff_stream, // TODO keep lifetime on this?
                nvtiff_decoder,
                decode_params,
                0, // image_id
            )
        };
        dbg!(status_check); // 4: NVTIFF_STATUS_TIFF_NOT_SUPPORTED; 2: NVTIFF_STATUS_INVALID_PARAMETER
        assert_eq!(status_check, nvtiffStatus_t::NVTIFF_STATUS_SUCCESS);

        // Step 3b: Allocate memory on device, get pointer, do the TIFF decoding
        let num_bytes: usize = self.file_info.image_width as usize // Width
            * self.file_info.image_height as usize // Height
            * (self.file_info.bits_per_pixel as usize / 8); // Bytes per pixel (e.g. 4 bytes for f32)
        let image_stream: CudaSlice<u8> = stream.alloc_zeros::<u8>(num_bytes).unwrap();
        let (image_ptr, _record): (u64, _) = image_stream.device_ptr(stream);
        let image_out_d = image_ptr as *mut c_void;
        let status_decode: u32 = unsafe {
            nvtiffDecodeImage(
                self.tiff_stream,
                nvtiff_decoder,
                decode_params,
                0, // image_id
                image_out_d,
                cuda_stream,
            )
        };
        dbg!(status_decode); // 4: NVTIFF_STATUS_TIFF_NOT_SUPPORTED; 8: NVTIFF_STATUS_INTERNAL_ERROR
        assert_eq!(status_decode, nvtiffStatus_t::NVTIFF_STATUS_SUCCESS);

        // todo!();

        image_stream.clone()
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use cudarc::driver::{CudaContext, CudaSlice, CudaStream};
    use object_store::parse_url;
    use url::Url;

    use crate::io::nvtiff::CudaCogReader;

    #[tokio::test]
    async fn test_cudacogreader_to_cuda() {
        let cog_url: &str =
            "https://github.com/rasterio/rasterio/raw/refs/tags/1.4.3/tests/data/float32.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();

        // let v = std::fs::read("images/float32.tif").unwrap();
        // let bytes = Bytes::copy_from_slice(&v);

        // Step 1b: Init CUDA stream on device (GPU)
        let ctx: Arc<CudaContext> = cudarc::driver::CudaContext::new(0).unwrap(); // Set on GPU:0
        let cuda_stream: Arc<CudaStream> = ctx.default_stream();

        let reader: CudaCogReader = CudaCogReader::new(&bytes);

        // Step 3b: Allocate memory on device, get pointer, do the TIFF decoding
        let num_bytes: usize = 3 * 2 * 4; // Width:3, Height:2, 4 bytes per f32 num

        // let image_stream: CudaSlice<u8> = cuda_stream.alloc_zeros::<u8>(num_bytes).unwrap();
        let slice: CudaSlice<u8> = reader.to_cuda(&cuda_stream);

        // todo!();

        // Step 2c: Transfer decoded bytes from device to host, and check results
        let mut image_out_h: Vec<u8> = vec![0; num_bytes];
        cuda_stream.memcpy_dtoh(&slice, &mut image_out_h).unwrap();
        dbg!(image_out_h.clone());
        let float_array: Vec<f32> = image_out_h
            // https://stackoverflow.com/questions/77388769/convert-vecu8-to-vecfloat-in-rust
            // .array_chunks::<4>()
            // .copied()
            .chunks_exact(4)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .map(f32::from_le_bytes)
            .collect();
        assert_eq!(float_array, vec![1.41, 1.23, 0.78, 0.32, -0.23, -1.88]);
    }
}
