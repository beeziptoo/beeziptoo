use thiserror::Error as ThisError;

mod rle;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Failed to decode at a runlength step")]
    RunLengthDecode,
}

impl From<rle::Error> for Error {
    fn from(value: rle::Error) -> Self {
        match value {
            rle::Error::RunLengthTruncated => Error::RunLengthDecode,
        }
    }
}

pub fn compress(data: &[u8]) -> Vec<u8> {
    rle::forward(data)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>, Error> {
    rle::reverse(data).map_err(|e| e.into())
}
