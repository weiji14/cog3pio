use std::ffi::c_void;
use std::sync::Arc;

use bytes::Bytes;
use cudarc::driver::{CudaSlice, CudaStream, DevicePtr};
use dlpark::SafeManagedTensorVersioned;
use nvtiff_sys::result::{NvTiffError, NvTiffStatusError};
use nvtiff_sys::{
    NvTiffResult, NvTiffResultCheck, nvtiffDecodeCheckSupported, nvtiffDecodeImage,
    nvtiffDecodeParams, nvtiffDecoder, nvtiffDecoderCreateSimple, nvtiffFileInfo, nvtiffStatus_t,
    nvtiffStream, nvtiffStreamCreate, nvtiffStreamGetFileInfo, nvtiffStreamParse,
};

/// Cloud-optimized GeoTIFF reader using [`nvTIFF`](https://developer.nvidia.com/nvtiff)
pub struct CudaCogReader {
    tiff_stream: *mut nvtiffStream,
    num_bytes: usize,
    cuda_slice: CudaSlice<u8>,
}

impl CudaCogReader {
    /// Create a new Cloud-optimized GeoTIFF decoder that decodes from a CUDA stream
    /// buffer
    ///
    /// # Errors
    /// Will return [`nvtiff_sys::result::NvTiffError::StatusError`] if nvTIFF failed to
    /// parse the TIFF data or metadata from the byte stream buffer.
    ///
    /// # Panics
    /// Will panic if [`CudaStream::alloc_zeros`] failed to allocate bytes on CUDA
    /// device memory, usually due to
    /// [`cudarc::driver::sys::cudaError_enum::CUDA_ERROR_OUT_OF_MEMORY`]
    pub fn new(byte_stream: &Bytes, cuda_stream: &Arc<CudaStream>) -> NvTiffResult<Self> {
        // Step 0: Init TIFF stream on host (CPU)
        let mut host_stream = std::mem::MaybeUninit::uninit();
        let mut tiff_stream: *mut nvtiffStream = host_stream.as_mut_ptr();

        let status_cpustream: nvtiffStatus_t::Type =
            unsafe { nvtiffStreamCreate(&raw mut tiff_stream) };
        dbg!(status_cpustream);
        status_cpustream.result()?;

        // Step 1: Parse the TIFF data from byte stream buffer
        let status_parse: u32 =
            unsafe { nvtiffStreamParse(byte_stream.as_ptr(), usize::MAX, tiff_stream) };
        dbg!(status_parse);
        status_parse.result()?;

        // Step 2: Extract file-level metadata information from the TIFF stream
        let mut file_info = nvtiffFileInfo::default();
        let status_fileinfo: u32 =
            unsafe { nvtiffStreamGetFileInfo(tiff_stream, &raw mut file_info) };
        dbg!(status_fileinfo);
        // dbg!(file_info);
        status_fileinfo.result()?;

        // Step 3b: Allocate memory on device, get pointer, do the TIFF decoding
        let num_bytes: usize = file_info.image_width as usize // Width
            * file_info.image_height as usize // Height
            * (file_info.bits_per_pixel as usize / 8); // Bytes per pixel (e.g. 4 bytes for f32)
        dbg!(num_bytes);
        let image_stream: CudaSlice<u8> =
            cuda_stream
                .alloc_zeros::<u8>(num_bytes)
                .unwrap_or_else(|err| {
                    panic!("Failed to allocate {num_bytes} bytes on CUDA device: {err}")
                });

        Ok(Self {
            tiff_stream,
            num_bytes,
            cuda_slice: image_stream,
        })
    }

    /// Decode GeoTIFF image to a [`CudaSlice`] (`Vec<u8>` on a CUDA device)
    ///
    /// # Errors
    ///
    /// Will raise [`nvtiff_sys::result::NvTiffError::StatusError`] if decoding failed
    /// due to e.g. TIFF stream not being supported by nvTIFF, missing
    /// nvCOMP/nvJPEG/nvJPEG2K libraries, etc.
    pub fn dlpack(&self) -> NvTiffResult<SafeManagedTensorVersioned> {
        // Step 1b: Init CUDA stream on device (GPU)
        let stream: &Arc<CudaStream> = self.cuda_slice.stream();
        let cuda_stream: *mut nvtiff_sys::CUstream_st = stream.cu_stream().cast::<_>();

        // Step 1c: Init decoder handle
        let mut decoder_handle = std::mem::MaybeUninit::zeroed();
        let mut nvtiff_decoder: *mut nvtiffDecoder = decoder_handle.as_mut_ptr();

        let status_decoder: u32 =
            unsafe { nvtiffDecoderCreateSimple(&raw mut nvtiff_decoder, cuda_stream) };
        dbg!(status_decoder);
        status_decoder.result()?;

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
        status_check.result()?;

        // Step 3b: Do the TIFF decoding to allocated device memory
        let (image_ptr, _record): (u64, _) = self.cuda_slice.device_ptr(stream);
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
        status_decode.result()?;

        // Create CudaSlice from pointer
        let cuslice: CudaSlice<u8> =
            unsafe { stream.upgrade_device_ptr(image_ptr, self.num_bytes) };
        // Put CudaSlice into DLPack tensor
        let tensor = SafeManagedTensorVersioned::new(cuslice)
            // TODO raise error from err string
            .map_err(|_| NvTiffError::StatusError(NvTiffStatusError::AllocatorFailure))?;

        Ok(tensor)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use cudarc::driver::{CudaContext, CudaSlice, CudaStream};
    use dlpark::SafeManagedTensorVersioned;
    use dlpark::ffi::DataType;
    use dlpark::prelude::TensorView;
    use object_store::parse_url;
    use url::Url;

    use crate::io::nvtiff::CudaCogReader;

    #[tokio::test]
    async fn cudacogreader_dlpack() {
        let cog_url: &str =
            "https://github.com/rasterio/rasterio/raw/refs/tags/1.4.3/tests/data/float32.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();

        // let v = std::fs::read("benches/float32.tif").unwrap();
        // let bytes = Bytes::copy_from_slice(&v);

        // Step 1: Init CUDA stream on device (GPU)
        let ctx: Arc<CudaContext> = cudarc::driver::CudaContext::new(0).unwrap(); // Set on GPU:0
        let cuda_stream: Arc<CudaStream> = ctx.default_stream();

        // Step 2: Do the TIFF decoding
        let cog: CudaCogReader = CudaCogReader::new(&bytes, &cuda_stream).unwrap();
        let tensor: SafeManagedTensorVersioned = cog.dlpack().unwrap();

        // assert_eq!(tensor.data_type(), &DataType::F32); // TODO should be f32 dtype
        assert_eq!(tensor.data_type(), &DataType::U8);
        // assert_eq!(tensor.shape(), [1, 2, 3]); // TODO should be 3D tensor
        assert_eq!(tensor.shape(), [24]);

        // Step 3: Transfer decoded bytes from device to host, and check results
        let mut image_out_h: Vec<u8> = vec![0; tensor.num_bytes()];
        let cuslice: CudaSlice<_> = unsafe {
            cuda_stream.upgrade_device_ptr(tensor.data_ptr() as u64, tensor.num_elements())
        };
        cuda_stream
            .memcpy_dtoh(&cuslice.clone(), &mut image_out_h)
            .unwrap();
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
