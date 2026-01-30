use exn::Result;

/// Error variants in the cog3pio library.
#[derive(Debug)]
#[non_exhaustive]
pub enum Cog3pioError {
    /// Error when decoding a TIFF.
    Decode {
        /// Message describing the error
        msg: String,
    },
    /// Error due to a feature not implemented in some library.
    Unimplemented {
        /// Name of library where feature is missing
        lib: &'static str,
        /// Message describing the error
        msg: String,
    },
}

impl std::fmt::Display for Cog3pioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cog3pioError::Decode { msg } => write!(f, "decode error: {msg}"),
            Cog3pioError::Unimplemented { lib, msg } => {
                write!(f, "unimplemented by {lib}: {msg}")
            }
        }
    }
}

impl std::error::Error for Cog3pioError {}

/// Result from a cog3pio API call
pub type Cog3pioResult<T> = Result<T, Cog3pioError>;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use object_store::parse_url;
    use url::Url;

    use crate::io::geotiff::{CogReader, shape_vec_to_tensor};

    #[test]
    fn decode_error() {
        let result = shape_vec_to_tensor((1, 2, 3), vec![0, 1, 2]);
        assert_eq!(
            unsafe { result.unwrap_err_unchecked().to_string() },
            "decode error: failed to convert vector of size 3 to shape (1, 2, 3)"
        );
    }

    #[tokio::test]
    async fn unimplemented_error() {
        let cog_url: &str =
            "https://github.com/image-rs/image-tiff/raw/v0.11.2/tests/images/tiled-cmyk-i8.tif";
        let tif_url = Url::parse(cog_url).unwrap();
        let (store, location) = parse_url(&tif_url).unwrap();

        let result = store.get(&location).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        let stream = Cursor::new(bytes);

        let mut cog = CogReader::new(stream).unwrap();
        let result = cog.num_samples();

        assert_eq!(
            result.unwrap_err().to_string(),
            "unimplemented by cog3pio: color type CMYK(8) not supported yet"
        );
    }
}
