use std::io::{Error, Read, Seek};

use dlpark::traits::InferDataType;
use dlpark::SafeManagedTensorVersioned;
use geo::AffineTransform;
use ndarray::{Array, Array1, Array3, ArrayView3, ArrayViewD};
use tiff::decoder::{Decoder, DecodingResult, Limits};
use tiff::tags::Tag;
use tiff::{ColorType, TiffError, TiffFormatError, TiffResult, TiffUnsupportedError};

/// Cloud-optimized GeoTIFF reader
pub struct CogReader<R: Read + Seek> {
    /// TIFF decoder
    decoder: Decoder<R>,
}

impl<R: Read + Seek> CogReader<R> {
    /// Create a new GeoTIFF decoder that decodes from a stream buffer
    pub fn new(stream: R) -> TiffResult<Self> {
        // Open TIFF stream with decoder
        let mut decoder = Decoder::new(stream)?;
        decoder = decoder.with_limits(Limits::unlimited());

        Ok(Self { decoder })
    }

    /// Decode GeoTIFF image to a [`dlpark::SafeManagedTensorVersioned`]
    pub fn dlpack(&mut self) -> TiffResult<SafeManagedTensorVersioned> {
        // Count number of bands
        let num_bands: usize = self.num_samples()?;
        // Get image dimensions
        let (width, height): (u32, u32) = self.decoder.dimensions()?;

        // Get image pixel data
        let decode_result = self.decoder.read_image()?;

        let shape = (num_bands, height as usize, width as usize);
        let tensor: SafeManagedTensorVersioned = match decode_result {
            DecodingResult::U8(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::U16(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::U32(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::U64(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::I8(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::I16(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::I32(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::I64(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::F32(img_data) => shape_vec_to_tensor(shape, img_data)?,
            DecodingResult::F64(img_data) => shape_vec_to_tensor(shape, img_data)?,
        };

        Ok(tensor)
    }

    /// Number of samples per pixel, also known as channels or bands
    fn num_samples(&mut self) -> TiffResult<usize> {
        let color_type = self.decoder.colortype()?;
        let num_bands: usize = match color_type {
            ColorType::Multiband {
                bit_depth: _,
                num_samples,
            } => num_samples as usize,
            ColorType::Gray(_) => 1,
            ColorType::RGB(_) => 3,
            _ => {
                return Err(TiffError::UnsupportedError(
                    TiffUnsupportedError::UnsupportedColorType(color_type),
                ))
            }
        };
        Ok(num_bands)
    }

    /// Affine transformation for 2D matrix extracted from TIFF tag metadata, used to transform
    /// image pixel (row, col) coordinates to and from geographic/projected (x, y) coordinates.
    ///
    /// ```text
    /// | x' |   | a b c | | x |
    /// | y' | = | d e f | | y |
    /// | 1  |   | 0 0 1 | | 1 |
    /// ```
    ///
    /// where (`x'` and `y'`) are world coordinates, and (`x`, `y`) are the pixel's
    /// image coordinates. Letters a to f represent:
    ///
    /// - `a` - width of a pixel (x-resolution)
    /// - `b` - row rotation (typically zero)
    /// - `c` - x-coordinate of the *center* of the upper-left pixel (x-origin)
    /// - `d` - column rotation (typically zero)
    /// - `e` - height of a pixel (y-resolution, typically negative)
    /// - `f` - y-coordinate of the *center* of the upper-left pixel (y-origin)
    ///
    /// References:
    /// - <https://docs.ogc.org/is/19-008r4/19-008r4.html#_coordinate_transformations>
    fn transform(&mut self) -> TiffResult<AffineTransform<f64>> {
        // Get x and y axis rotation (not yet implemented)
        let (x_rotation, y_rotation): (f64, f64) =
            match self.decoder.get_tag_f64_vec(Tag::ModelTransformationTag) {
                Ok(_model_transformation) => unimplemented!("Non-zero rotation is not handled yet"),
                Err(_) => (0.0, 0.0),
            };

        // Get pixel size in x and y direction
        let pixel_scale: Vec<f64> = self.decoder.get_tag_f64_vec(Tag::ModelPixelScaleTag)?;
        let [x_scale, y_scale, _z_scale] = pixel_scale[0..3] else {
            return Err(TiffError::FormatError(TiffFormatError::InvalidTag));
        };

        // Get x and y coordinates of upper left pixel
        let tie_points: Vec<f64> = self.decoder.get_tag_f64_vec(Tag::ModelTiepointTag)?;
        let [_i, _j, _k, x_origin, y_origin, _z_origin] = tie_points[0..6] else {
            return Err(TiffError::FormatError(TiffFormatError::InvalidTag));
        };

        // Create affine transformation matrix
        let transform = AffineTransform::new(
            x_scale, x_rotation, x_origin, y_rotation, -y_scale, y_origin,
        );

        Ok(transform)
    }

    /// Get list of x and y coordinates
    pub fn xy_coords(&mut self) -> TiffResult<(Array1<f64>, Array1<f64>)> {
        let transform = self.transform()?; // affine transformation matrix

        // Get spatial resolution in x and y dimensions
        let x_res: &f64 = &transform.a();
        let y_res: &f64 = &transform.e();

        // Get xy coordinate of the center of the top left pixel
        let x_origin: &f64 = &(transform.xoff() + x_res / 2.0);
        let y_origin: &f64 = &(transform.yoff() + y_res / 2.0);

        // Get number of pixels along the x and y dimensions
        let (x_pixels, y_pixels): (u32, u32) = self.decoder.dimensions()?;

        // Get xy coordinate of the center of the bottom right pixel
        let x_end: f64 = x_origin + x_res * x_pixels as f64;
        let y_end: f64 = y_origin + y_res * y_pixels as f64;

        // Get array of x-coordinates and y-coordinates
        let x_coords = Array::range(x_origin.to_owned(), x_end, x_res.to_owned());
        let y_coords = Array::range(y_origin.to_owned(), y_end, y_res.to_owned());

        Ok((x_coords, y_coords))
    }
}

/// Convert Vec<T> into an Array3<T> with a shape of (channels, height, width), and then
/// output it as a DLPack tensor.
fn shape_vec_to_tensor<T: InferDataType>(
    shape: (usize, usize, usize),
    vec: Vec<T>,
) -> TiffResult<SafeManagedTensorVersioned> {
    let array_data = Array3::from_shape_vec(shape, vec)
        .map_err(|_| TiffFormatError::InconsistentSizesEncountered)?;
    let tensor = SafeManagedTensorVersioned::new(array_data)
        .map_err(|err| TiffError::IoError(Error::other(err.to_string())))?;
    Ok(tensor)
}

/// Synchronously read a GeoTIFF file into an [`ndarray::Array`]
pub fn read_geotiff<T: InferDataType + Clone, R: Read + Seek>(stream: R) -> TiffResult<Array3<T>> {
    // Open TIFF stream with decoder
    let mut reader = CogReader::new(stream)?;

    // Decode TIFF into DLPack
    let tensor: SafeManagedTensorVersioned = reader.dlpack()?;

    // Count number of bands
    let num_bands: usize = reader.num_samples()?;
    // Get image dimensions
    let (width, height): (u32, u32) = reader.decoder.dimensions()?;

    // Convert DLPack tensor to ndarray
    let view = ArrayViewD::<T>::try_from(&tensor)
        .map_err(|err| TiffError::IoError(Error::other(err.to_string())))?;
    let array: ArrayView3<T> = view
        .into_shape_with_order((num_bands, height as usize, width as usize))
        .map_err(|err| TiffError::IoError(Error::other(err.to_string())))?;

    Ok(array.to_owned())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};

    use dlpark::ffi::DataType;
    use dlpark::prelude::TensorView;
    use dlpark::SafeManagedTensorVersioned;
    use geo::AffineTransform;
    use ndarray::{s, Array3};
    use object_store::parse_url;
    use tempfile::tempfile;
    use tiff::encoder::{colortype, TiffEncoder};
    use url::Url;

    use crate::io::geotiff::{read_geotiff, CogReader};

    #[test]
    fn test_read_geotiff() {
        // Generate some data
        let mut image_data = Vec::new();
        for y in 0..10 {
            for x in 0..20 {
                let val = y + x;
                image_data.push(val as f32);
            }
        }

        // Write a BigTIFF file
        let mut file = tempfile().unwrap();
        let mut bigtiff = TiffEncoder::new_big(&mut file).unwrap();
        bigtiff
            .write_image::<colortype::Gray32Float>(20, 10, &image_data) // width, height, data
            .unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        // Read a BigTIFF file
        let arr: Array3<f32> = read_geotiff(file).unwrap();
        assert_eq!(arr.ndim(), 3);
        assert_eq!(arr.dim(), (1, 10, 20)); // (channels, height, width)
        let first_band = arr.slice(s![0, .., ..]);
        assert_eq!(first_band.nrows(), 10); // y-axis
        assert_eq!(first_band.ncols(), 20); // x-axis
        assert_eq!(arr.mean(), Some(14.0));
    }

    #[tokio::test]
    async fn test_read_geotiff_multi_band() {
        let cog_url: &str =
            "https://github.com/locationtech/geotrellis/raw/v3.7.1/raster/data/one-month-tiles-multiband/result.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        let stream = Cursor::new(bytes);

        let array: Array3<f32> = read_geotiff(stream).unwrap();

        assert_eq!(array.dim(), (2, 512, 512));
        assert_eq!(array.mean(), Some(225.17654));
    }

    #[tokio::test]
    async fn test_read_geotiff_uint16_dtype() {
        let cog_url: &str =
            "https://github.com/OSGeo/gdal/raw/v3.9.2/autotest/gcore/data/uint16.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        let stream = Cursor::new(bytes);

        let array: Array3<u16> = read_geotiff(stream).unwrap();

        assert_eq!(array.dim(), (1, 20, 20));
        assert_eq!(array.mean(), Some(126));
    }

    #[tokio::test]
    async fn test_cogreader_dlpack() {
        let cog_url: &str = "https://github.com/rasterio/rasterio/raw/1.3.9/tests/data/float32.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        let stream = Cursor::new(bytes);

        let mut cog = CogReader::new(stream).unwrap();
        let tensor: SafeManagedTensorVersioned = cog.dlpack().unwrap();

        assert_eq!(tensor.shape(), [1, 2, 3]);
        assert_eq!(tensor.data_type(), &DataType::F32);
        let values: Vec<f32> = tensor
            .to_vec()
            .chunks_exact(4)
            .map(TryInto::try_into)
            .map(Result::unwrap)
            .map(f32::from_le_bytes)
            .collect();
        assert_eq!(values, vec![1.41, 1.23, 0.78, 0.32, -0.23, -1.88]);
    }

    #[tokio::test]
    async fn test_cogreader_num_samples() {
        let cog_url: &str = "https://github.com/developmentseed/titiler/raw/refs/tags/0.22.2/src/titiler/mosaic/tests/fixtures/TCI.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        let stream = Cursor::new(bytes);

        let mut cog = CogReader::new(stream).unwrap();
        assert_eq!(cog.num_samples().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_cogreader_transform() {
        let cog_url: &str =
            "https://github.com/cogeotiff/rio-tiler/raw/6.4.0/tests/fixtures/cog_nodata_nan.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        let stream = Cursor::new(bytes);

        let mut cog = CogReader::new(stream).unwrap();
        let transform = cog.transform().unwrap();

        assert_eq!(
            transform,
            AffineTransform::new(200.0, 0.0, 499980.0, 0.0, -200.0, 5300040.0)
        );
    }
}
