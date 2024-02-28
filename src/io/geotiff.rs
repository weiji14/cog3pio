use std::io::{Read, Seek};

use ndarray::{Array2, ShapeError};
use tiff::decoder::{DecodingResult, Limits};

/// Synchronously read a GeoTIFF file into an [`ndarray::Array`]
pub fn read_geotiff<R: Read + Seek>(stream: R) -> Result<Array2<f32>, ShapeError> {
    // Open TIFF stream with decoder
    let mut decoder = tiff::decoder::Decoder::new(stream).expect("Cannot create tiff decoder");
    decoder = decoder.with_limits(Limits::unlimited());

    // Get image dimensions
    let dimensions: (u32, u32) = decoder.dimensions().expect("Cannot parse image dimensions");
    let width = dimensions.0 as usize;
    let height = dimensions.1 as usize;

    // Get image pixel data
    let DecodingResult::F32(img_data) = decoder.read_image().expect("Cannot decode tiff image")
    else {
        panic!("Cannot read band data")
    };

    // Put image pixel data into an ndarray
    let vec_data = Array2::from_shape_vec((height, width), img_data)?;

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
        for x in 0..20 {
            for y in 0..20 {
                let val = x + y;
                image_data.push(val as f32);
            }
        }

        // Write a BigTIFF file
        let mut file = tempfile().unwrap();
        let mut bigtiff = TiffEncoder::new_big(&mut file).unwrap();
        bigtiff
            .write_image::<colortype::Gray32Float>(20, 20, &image_data)
            .unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        // Read a BigTIFF file
        let arr = read_geotiff(file).unwrap();
        assert_eq!(arr.dim(), (20, 20));
        assert_eq!(arr.mean(), Some(19.0));
    }
}
