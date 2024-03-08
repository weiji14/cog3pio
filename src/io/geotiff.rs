use std::io::{Read, Seek};

use ndarray::Array2;
use tiff::decoder::{DecodingResult, Limits};
use tiff::{TiffError, TiffFormatError};

/// Synchronously read a GeoTIFF file into an [`ndarray::Array`]
pub fn read_geotiff<R: Read + Seek>(stream: R) -> Result<Array2<f32>, TiffError> {
    // Open TIFF stream with decoder
    let mut decoder = tiff::decoder::Decoder::new(stream)?;
    decoder = decoder.with_limits(Limits::unlimited());

    // Get image dimensions
    let (height, width): (u32, u32) = decoder.dimensions()?;

    // Get image pixel data
    let DecodingResult::F32(img_data) = decoder.read_image()? else {
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
            .write_image::<colortype::Gray32Float>(20, 10, &image_data)
            .unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        // Read a BigTIFF file
        let arr = read_geotiff(file).unwrap();
        assert_eq!(arr.ndim(), 2);
        assert_eq!(arr.dim(), (20, 10));  // (height, width)
        assert_eq!(arr.nrows(), 20);  // y-axis
        assert_eq!(arr.ncols(), 10);  // x-axis
        assert_eq!(arr.mean(), Some(14.0));
    }
}
