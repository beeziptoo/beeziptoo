//! beeziptoo
//!
//! Because we wanted to implement `bzip2`, too.
use std::io::{self, Cursor, Read};

mod burrows_wheeler;
mod move_to_front;
mod rle1;
mod rle2;

/// These are the possible errors that can occur during compression.
#[derive(Debug, thiserror::Error)]
pub enum CompressError {
    /// An IO error occurred.
    #[error("I/O error: {0}")]
    IOError(io::Error),
}

impl From<io::Error> for CompressError {
    fn from(value: io::Error) -> Self {
        CompressError::IOError(value)
    }
}

/// These are the possible errors that can occur during decompression.
#[derive(Debug, thiserror::Error)]
pub enum DecompressError {
    /// An IO error occurred.
    #[error("I/O error: {0}")]
    IOError(io::Error),
    /// The runlength decoder encountered an invalid input.
    #[error("Failed to decode at a runlength step")]
    RunLengthDecode,
    /// The burrows-wheeler decoder encountered an invalid input.
    #[error("Failed to decode at a burrows-wheeler step")]
    BurrowsWheelerDecode,
}

impl From<io::Error> for DecompressError {
    fn from(value: io::Error) -> Self {
        DecompressError::IOError(value)
    }
}

impl From<rle1::Error> for DecompressError {
    fn from(value: rle1::Error) -> Self {
        match value {
            rle1::Error::RunLengthInvalid(_) => DecompressError::RunLengthDecode,
            rle1::Error::RunLengthTruncated => DecompressError::RunLengthDecode,
        }
    }
}

impl From<burrows_wheeler::DecodeError> for DecompressError {
    fn from(_value: burrows_wheeler::DecodeError) -> Self {
        DecompressError::BurrowsWheelerDecode
    }
}

/// Compress the given data.
pub fn compress<R>(mut data: R) -> Result<impl Read, CompressError>
where
    R: Read,
{
    let mut all_data = vec![];
    data.read_to_end(&mut all_data)?;

    let rle_data = rle1::encode(&all_data);
    let burrows_wheeler_data = burrows_wheeler::encode(&rle_data);
    let move_to_front_data = move_to_front::encode(&burrows_wheeler_data);
    let _rle2_data = rle2::encode(&move_to_front_data);

    let output = move_to_front_data;
    let cursor = Cursor::new(output);

    Ok(cursor)
}

/// Decompress the given data.
///
/// # Errors
///
/// This function is failable since it is possible the given data isn't a valid `bzip2` archive.
pub fn decompress<R>(mut data: R) -> Result<impl Read, DecompressError>
where
    R: Read,
{
    let mut all_data = vec![];
    data.read_to_end(&mut all_data)?;

    // TODO: Pass in some real symbols.
    let _un_rle2 = rle2::decode(&[]);
    let un_move_to_front_data = move_to_front::decode(&all_data);
    let un_burrows_wheeler_data = burrows_wheeler::decode(&un_move_to_front_data)?;
    let un_rle_data = rle1::decode(&un_burrows_wheeler_data)?;

    let cursor = Cursor::new(un_rle_data);

    Ok(cursor)
}
