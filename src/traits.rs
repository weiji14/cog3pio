use std::error::Error;
use std::marker::Sized;

// use exn::Exn;
use geo::AffineTransform;
use ndarray::Array1;

// use crate::Cog3pioResult;

/// A trait that provides the Affine transformation matrix for a TIFF struct.
pub trait Transform {
    /// The error type when returning the Affine transform fails
    type Err: Error + Send + Sync;
    /// Affine transformation for 2D matrix extracted from TIFF tag metadata, used to
    /// transform image pixel (row, col) coordinates to and from geographic/projected
    /// (x, y) coordinates.
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
    ///
    /// # Errors
    ///
    /// Will return [`Self::Err`] if the Affine transformation matrix cannot be created
    /// from the underlying TIFF tag metadata, perhaps due to invalid or unimplemented
    /// parsing of the `ModelPixelScaleTag`, `ModelTiepointTag` or
    /// `ModelTransformationTag`.
    fn transform(self) -> exn::Result<AffineTransform, Self::Err>;

    /// Get list of x and y coordinates
    ///
    /// Determined based on an [`AffineTransform`] matrix built from the
    /// `ModelPixelScaleTag` and `ModelTiepointTag`, or `ModelTransformationTag`.
    ///
    /// # Errors
    ///
    /// Will return [`Self::Err`] if the TIFF file is missing tags required to build an
    /// Affine transformation matrix, or height/width of the TIFF image cannot be
    /// determined.
    fn xy_coords(self) -> exn::Result<(Array1<f64>, Array1<f64>), Self::Err>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
