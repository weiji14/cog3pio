use std::io::{Read, Seek};

use geo::AffineTransform;
use ndarray::Array2;
use tiff::decoder::{Decoder, DecodingResult, Limits};
use tiff::tags::Tag;
use tiff::{TiffError, TiffFormatError, TiffResult};

/// Cloud-optimized GeoTIFF reader
struct CogReader<R: Read + Seek> {
    decoder: Decoder<R>,
}

impl<R: Read + Seek> CogReader<R> {
    /// Create a new GeoTIFF decoder that decodes from a stream buffer
    fn new(stream: R) -> TiffResult<Self> {
        // Open TIFF stream with decoder
        let mut decoder = Decoder::new(stream)?;
        decoder = decoder.with_limits(Limits::unlimited());

        Ok(Self { decoder })
    }

    /// Decode GeoTIFF image to a Vec
    fn as_vec(&mut self) -> TiffResult<Vec<f32>> {
        let decode_result = self.decoder.read_image()?;
        let image_data: Vec<f32> = match decode_result {
            DecodingResult::F32(img_data) => img_data,
            _ => unimplemented!("Data types other than float32 are not yet supported."),
        };
        Ok(image_data)
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
}

/// Synchronously read a GeoTIFF file into an [`ndarray::Array`]
pub fn read_geotiff<R: Read + Seek>(stream: R) -> TiffResult<Array2<f32>> {
    // Open TIFF stream with decoder
    let mut reader = CogReader::new(stream)?;

    // Get image dimensions
    let (width, height): (u32, u32) = reader.decoder.dimensions()?;

    // Get image pixel data
    let img_data: Vec<f32> = reader.as_vec()?;

    // Put image pixel data into an ndarray
    let vec_data = Array2::from_shape_vec((height as usize, width as usize), img_data)
        .map_err(|_| TiffFormatError::InvalidDimensions(height, width))?;

    Ok(vec_data)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};

    use geo::AffineTransform;
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
        assert_eq!(arr.ndim(), 2);
        assert_eq!(arr.dim(), (10, 20)); // (height, width)
        assert_eq!(arr.nrows(), 10); // y-axis
        assert_eq!(arr.ncols(), 20); // x-axis
        assert_eq!(arr.mean(), Some(14.0));
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
