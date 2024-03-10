use std::io::{Read, Seek};

use ndarray::Array2;
use tiff::decoder::{Decoder, DecodingResult, Limits};
use tiff::{TiffFormatError, TiffResult};

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
    use std::io::{Seek, SeekFrom};

    use tempfile::tempfile;
    use tiff::encoder::{colortype, TiffEncoder};

    use crate::io::geotiff::read_geotiff;

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
}
