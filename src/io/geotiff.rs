use std::io::{Read, Seek};

use geo::AffineTransform;
use ndarray::{Array, Array1, Array3};
use tiff::decoder::{Decoder, DecodingResult, Limits};
use tiff::tags::Tag;
use tiff::{ColorType, TiffError, TiffFormatError, TiffResult, TiffUnsupportedError};

/// Cloud-optimized GeoTIFF reader
pub(crate) struct CogReader<R: Read + Seek> {
    /// TIFF decoder
    pub decoder: Decoder<R>,
}

impl<R: Read + Seek> CogReader<R> {
    /// Create a new GeoTIFF decoder that decodes from a stream buffer
    pub fn new(stream: R) -> TiffResult<Self> {
        // Open TIFF stream with decoder
        let mut decoder = Decoder::new(stream)?;
        decoder = decoder.with_limits(Limits::unlimited());

        Ok(Self { decoder })
    }

    /// Decode GeoTIFF image to an [`ndarray::Array`]
    pub fn ndarray(&mut self) -> TiffResult<Array3<f32>> {
        // Count number of bands
        let color_type = self.decoder.colortype()?;
        let num_bands: usize = match color_type {
            ColorType::Multiband {
                bit_depth: _,
                num_samples,
            } => num_samples as usize,
            ColorType::Gray(_) => 1,
            _ => {
                return Err(TiffError::UnsupportedError(
                    TiffUnsupportedError::UnsupportedColorType(color_type),
                ))
            }
        };

        // Get image dimensions
        let (width, height): (u32, u32) = self.decoder.dimensions()?;

        // Get image pixel data
        let decode_result = self.decoder.read_image()?;
        let image_data: Vec<f32> = match decode_result {
            DecodingResult::F32(img_data) => img_data,
            _ => {
                return Err(TiffError::UnsupportedError(
                    TiffUnsupportedError::UnsupportedDataType,
                ))
            }
        };

        // Put image pixel data into an ndarray
        let array_data =
            Array3::from_shape_vec((num_bands, height as usize, width as usize), image_data)
                .map_err(|_| TiffFormatError::InconsistentSizesEncountered)?;

        Ok(array_data)
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
    /// where (`x'` and `y'`) are world coordinates, and (`x`, `y) are the pixel's image
    /// coordinates. Letters a to f represent:
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

        let x_origin: &f64 = transform.xoff();
        let y_origin: &f64 = transform.yoff();

        let x_res: &f64 = transform.a();
        let y_res: &f64 = transform.e();

        let (x_pixels, y_pixels): (u32, u32) = self.decoder.dimensions()?;

        let x_end: f64 = x_origin + x_res * x_pixels as f64;
        let y_end: f64 = y_origin + y_res * y_pixels as f64;

        let x_coords = Array::range(x_origin.to_owned(), x_end, x_res.to_owned());
        let y_coords = Array::range(y_origin.to_owned(), y_end, y_res.to_owned());

        Ok((x_coords, y_coords))
    }
}

/// Synchronously read a GeoTIFF file into an [`ndarray::Array`]
pub fn read_geotiff<R: Read + Seek>(stream: R) -> TiffResult<Array3<f32>> {
    // Open TIFF stream with decoder
    let mut reader = CogReader::new(stream)?;

    // Decode TIFF into ndarray
    let array_data: Array3<f32> = reader.ndarray()?;

    Ok(array_data)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};

    use geo::AffineTransform;
    use ndarray::{array, s};
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
        let arr = read_geotiff(file).unwrap();
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

        let mut reader = CogReader::new(stream).unwrap();
        let array = reader.ndarray().unwrap();

        assert_eq!(array.dim(), (2, 512, 512));
        assert_eq!(array.mean(), Some(225.17654));
    }

    #[tokio::test]
    async fn test_cogreader_ndarray() {
        let cog_url: &str = "https://github.com/rasterio/rasterio/raw/1.3.9/tests/data/float32.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        let stream = Cursor::new(bytes);

        let mut reader = CogReader::new(stream).unwrap();
        let array = reader.ndarray().unwrap();

        assert_eq!(array.shape(), [1, 2, 3]);
        assert_eq!(array, array![[[1.41, 1.23, 0.78], [0.32, -0.23, -1.88]]])
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

        let mut reader = CogReader::new(stream).unwrap();
        let transform = reader.transform().unwrap();

        assert_eq!(
            transform,
            AffineTransform::new(200.0, 0.0, 499980.0, 0.0, -200.0, 5300040.0)
        );
    }
}
