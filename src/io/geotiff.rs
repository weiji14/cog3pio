use ndarray::{Array2, ShapeError};
use std::fs::File;

use tiff::decoder::{DecodingResult, Limits};

/// Read a GeoTIFF file to an [`ndarray::Array`]
pub fn read_geotiff(path: &str) -> Result<Array2<f32>, ShapeError> {
    // Open TIFF file with decoder
    let file = File::open(path).expect("Cannot find GeoTIFF file");
    let mut decoder = tiff::decoder::Decoder::new(file).expect("Cannot create tiff decoder");
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
