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

    // Affine transformation
    fn transform(&mut self) -> TiffResult<AffineTransform<f64>> {
        // Get pixel size in x and y direction
        let pixel_scale: Vec<f64> = self.decoder.get_tag_f64_vec(Tag::ModelPixelScaleTag)?;
        let [x_scale, y_scale, _z_scale] = pixel_scale[0..3] else {
            return Err(TiffError::FormatError(TiffFormatError::InvalidTag));
        };

        // Get x and y coordinates of upper left pixel
        let tie_points: Vec<f64> = self.decoder.get_tag_f64_vec(Tag::ModelTiepointTag)?;
        let [_i, _j, _k, origin_x, origin_y, _origin_z] = tie_points[0..6] else {
            return Err(TiffError::FormatError(TiffFormatError::InvalidTag));
        };

        // Create affine transformation matrix
        let transform = AffineTransform::new(x_scale, 0.0, origin_x, 0.0, -y_scale, origin_y);

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
    let DecodingResult::F32(img_data) = reader.decoder.read_image()? else {
        panic!("Cannot read band data")
    };

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
    async fn test_get_transform() {
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
